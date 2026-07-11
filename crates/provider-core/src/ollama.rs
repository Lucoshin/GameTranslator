use serde::{Deserialize, Serialize};

use crate::{
    map_response_status, parse_structured_content, ProviderError, TranslationProvider,
    TranslationRequest, TranslationResponse,
};

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl OllamaProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl TranslationProvider for OllamaProvider {
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
            .post(format!("{}/api/chat", self.base_url))
            .json(&OllamaRequest {
                model: &request.model,
                stream: false,
                format: "json",
                messages: [Message {
                    role: "user",
                    content: &content,
                }],
            })
            .send()
            .map_err(|error| ProviderError::Transport(error.to_string()))?;
        map_response_status(response.status())?;
        let envelope: OllamaResponse = response
            .json()
            .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        parse_structured_content(&envelope.message.content)
    }
}

#[derive(Serialize)]
struct PromptPayload<'a> {
    task: &'a str,
    target_language: &'a str,
    segments: &'a [crate::TranslationInput],
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    stream: bool,
    format: &'a str,
    messages: [Message<'a>; 1],
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}
