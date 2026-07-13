mod adapters;
mod content;
mod credentials;
mod engine;
mod error;
mod models;
mod patch;
mod provider_factory;
mod translation_policy;

pub use adapters::AdapterRegistry;
pub use content::{detect_content, export_content, extract_content};
pub use credentials::{
    CredentialError, CredentialStore, InMemoryCredentialStore, ProviderConfiguration,
    WindowsCredentialStore,
};
pub use engine::{detect_game, engine_name, extract_game};
pub use error::{AppError, AppErrorCode};
pub use models::TaskStatus;
pub use patch::{PatchError, PatchFile, PatchManifest, PatchPlan};
pub use provider_factory::{ProviderFactory, ProviderSettings, DEFAULT_PROVIDER_CREDENTIAL};
pub use translation_policy::PerformanceSettings;

#[must_use]
pub const fn product_name() -> &'static str {
    "GameTranslator"
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_the_product_name() {
        assert_eq!(super::product_name(), "GameTranslator");
    }
}
