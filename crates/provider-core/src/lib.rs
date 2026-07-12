mod ollama;
mod openai_compatible;

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

pub use ollama::OllamaProvider;
pub use openai_compatible::OpenAiCompatibleProvider;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TranslationInput {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TranslationRequest {
    pub model: String,
    pub source_language: String,
    pub target_language: String,
    pub segments: Vec<TranslationInput>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TranslationOutput {
    pub id: String,
    #[serde(
        alias = "translated",
        alias = "translated_text",
        alias = "translatedText"
    )]
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TranslationResponse {
    pub translations: Vec<TranslationOutput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderError {
    RateLimited,
    HttpStatus(u16),
    Transport(String),
    InvalidResponse(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited => {
                formatter.write_str("translation provider rate limited the request")
            }
            Self::HttpStatus(status) => {
                write!(formatter, "translation provider returned HTTP {status}")
            }
            Self::Transport(message) => {
                write!(formatter, "translation provider request failed: {message}")
            }
            Self::InvalidResponse(message) => {
                write!(formatter, "invalid translation response: {message}")
            }
        }
    }
}

impl Error for ProviderError {}

pub trait TranslationProvider: Send + Sync {
    /// Translates one structured batch.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] for transport, HTTP, or response validation failures.
    fn translate(&self, request: &TranslationRequest)
        -> Result<TranslationResponse, ProviderError>;
}

pub(crate) fn parse_structured_content(
    content: &str,
) -> Result<TranslationResponse, ProviderError> {
    serde_json::from_str(content).map_err(|error| ProviderError::InvalidResponse(error.to_string()))
}

pub(crate) fn map_response_status(status: reqwest::StatusCode) -> Result<(), ProviderError> {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        Err(ProviderError::RateLimited)
    } else if status.is_success() {
        Ok(())
    } else {
        Err(ProviderError::HttpStatus(status.as_u16()))
    }
}
