use serde::{Deserialize, Serialize};

use crate::{
    map_response_status, parse_structured_content, ProviderError, TranslationProvider,
    TranslationRequest, TranslationResponse,
};

pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: String,
    client: reqwest::blocking::Client,
    user_id: Option<String>,
}

const TRANSLATION_SYSTEM_PROMPT: &str = "You are a game localization translator. Translate every segment into the requested target language, preserve every <ph> tag exactly, and return only a JSON object with a translations array containing the original ids and translated text.";

impl OpenAiCompatibleProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            client: reqwest::blocking::Client::builder()
                .pool_max_idle_per_host(128)
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .tcp_keepalive(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            user_id: None,
        }
    }

    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

impl TranslationProvider for OpenAiCompatibleProvider {
    fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, ProviderError> {
        let content = serde_json::to_string(&PromptPayload {
            source_language: &request.source_language,
            target_language: &request.target_language,
            segments: &request.segments,
        })
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .timeout(std::time::Duration::from_secs(120))
            .bearer_auth(&self.api_key)
            .json(&OpenAiRequest {
                model: &request.model,
                messages: [
                    Message {
                        role: "system",
                        content: TRANSLATION_SYSTEM_PROMPT,
                    },
                    Message {
                        role: "user",
                        content: &content,
                    },
                ],
                response_format: ResponseFormat {
                    kind: "json_object",
                },
                user_id: self.user_id.as_deref(),
            })
            .send()
            .map_err(|error| ProviderError::Transport(error.to_string()))?;
        map_response_status(response.status())?;
        let envelope: OpenAiResponse = response
            .json()
            .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        let content = envelope
            .choices
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("missing choices[0]".into()))?
            .message
            .content
            .as_str();
        parse_structured_content(content)
    }
}

#[derive(Serialize)]
struct PromptPayload<'a> {
    source_language: &'a str,
    target_language: &'a str,
    segments: &'a [crate::TranslationInput],
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: [Message<'a>; 2],
    response_format: ResponseFormat<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<&'a str>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ResponseFormat<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}
