use std::{collections::HashMap, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfiguration {
    pub id: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialError(String);

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CredentialError {}

pub trait CredentialStore {
    /// Stores a secret under a provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`CredentialError`] when the platform credential backend rejects the write.
    fn set(&mut self, provider_id: &str, secret: &str) -> Result<(), CredentialError>;

    /// Reads a secret without exposing it through project configuration.
    ///
    /// # Errors
    ///
    /// Returns [`CredentialError`] when the platform credential backend cannot be queried.
    fn get(&self, provider_id: &str) -> Result<Option<String>, CredentialError>;

    /// Removes a stored provider secret.
    ///
    /// # Errors
    ///
    /// Returns [`CredentialError`] when the platform credential backend rejects the deletion.
    fn delete(&mut self, provider_id: &str) -> Result<(), CredentialError>;
}

#[derive(Default)]
pub struct InMemoryCredentialStore {
    credentials: HashMap<String, String>,
}

impl CredentialStore for InMemoryCredentialStore {
    fn set(&mut self, provider_id: &str, secret: &str) -> Result<(), CredentialError> {
        self.credentials
            .insert(provider_id.to_owned(), secret.to_owned());
        Ok(())
    }

    fn get(&self, provider_id: &str) -> Result<Option<String>, CredentialError> {
        Ok(self.credentials.get(provider_id).cloned())
    }

    fn delete(&mut self, provider_id: &str) -> Result<(), CredentialError> {
        self.credentials.remove(provider_id);
        Ok(())
    }
}

pub struct WindowsCredentialStore {
    service: String,
}

impl WindowsCredentialStore {
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, provider_id: &str) -> Result<keyring::Entry, CredentialError> {
        keyring::Entry::new(&self.service, provider_id)
            .map_err(|error| CredentialError(error.to_string()))
    }
}

impl CredentialStore for WindowsCredentialStore {
    fn set(&mut self, provider_id: &str, secret: &str) -> Result<(), CredentialError> {
        self.entry(provider_id)?
            .set_password(secret)
            .map_err(|error| CredentialError(error.to_string()))
    }

    fn get(&self, provider_id: &str) -> Result<Option<String>, CredentialError> {
        match self.entry(provider_id)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(CredentialError(error.to_string())),
        }
    }

    fn delete(&mut self, provider_id: &str) -> Result<(), CredentialError> {
        match self.entry(provider_id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(CredentialError(error.to_string())),
        }
    }
}
