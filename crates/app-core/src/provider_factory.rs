use game_translator_provider_core::{
    OllamaProvider, OpenAiCompatibleProvider, TranslationProvider,
};

use crate::{AppError, AppErrorCode, CredentialStore};

pub const DEFAULT_PROVIDER_CREDENTIAL: &str = "default-provider";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSettings {
    pub kind: String,
    pub base_url: String,
    pub model: String,
}

impl ProviderSettings {
    #[must_use]
    pub fn new(
        kind: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }
}

pub struct ProviderFactory<'a> {
    credentials: &'a dyn CredentialStore,
}

impl<'a> ProviderFactory<'a> {
    #[must_use]
    pub const fn new(credentials: &'a dyn CredentialStore) -> Self {
        Self { credentials }
    }

    /// # Errors
    /// Returns a structured application error when a remote provider has no saved credential.
    pub fn create(
        &self,
        settings: &ProviderSettings,
    ) -> Result<Box<dyn TranslationProvider>, AppError> {
        if settings.kind == "ollama" {
            return Ok(Box::new(OllamaProvider::new(&settings.base_url)));
        }
        let api_key = self
            .credentials
            .get(DEFAULT_PROVIDER_CREDENTIAL)
            .map_err(|error| AppError::new(AppErrorCode::StorageFailure, error.to_string()))?
            .ok_or_else(|| AppError::new(AppErrorCode::CredentialMissing, "尚未保存 API Key"))?;
        let provider = OpenAiCompatibleProvider::new(&settings.base_url, api_key);
        if settings.base_url.contains("deepseek") || settings.model.starts_with("deepseek-") {
            Ok(Box::new(provider.with_user_id("game-translator-desktop")))
        } else {
            Ok(Box::new(provider))
        }
    }
}
