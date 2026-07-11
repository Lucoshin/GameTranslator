use serde::{Deserialize, Serialize};

use crate::{
    map_response_status, parse_structured_content, ProviderError, TranslationProvider,
    TranslationRequest, TranslationResponse,
};

pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: String,
    client: reqwest::blocking::Client,
}

impl OpenAiCompatibleProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl TranslationProvider for OpenAiCompatibleProvider {
    fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, ProviderError> {
        let content = serde_json::to_string(&PromptPayload {
            task: "Translate every segment into the target language. Preserve all <ph> tags exactly and return only the requested JSON object.",
            target_language: &request.target_language,
            segments: &request.segments,
        })
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&OpenAiRequest {
                model: &request.model,
                messages: [Message {
                    role: "user",
                    content: &content,
                }],
                response_format: ResponseFormat {
                    kind: "json_object",
                },
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
    task: &'a str,
    target_language: &'a str,
    segments: &'a [crate::TranslationInput],
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: [Message<'a>; 1],
    response_format: ResponseFormat<'a>,
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
