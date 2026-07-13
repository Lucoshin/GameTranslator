use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    UnsupportedSource,
    MissingRequiredFile,
    InvalidData,
    Io,
    CredentialMissing,
    ProviderRateLimited,
    ProviderFailure,
    InvalidProviderResponse,
    SourceChanged,
    QaBlocked,
    InvalidTaskTransition,
    StorageFailure,
    Unexpected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppError {
    code: AppErrorCode,
    message: String,
}

impl AppError {
    #[must_use]
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> AppErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for AppError {}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::new(AppErrorCode::Unexpected, message)
    }
}
