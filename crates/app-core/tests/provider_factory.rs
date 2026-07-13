use game_translator_app_core::{
    AppErrorCode, InMemoryCredentialStore, ProviderFactory, ProviderSettings,
};

#[test]
fn provider_factory_requires_credentials_only_for_remote_providers() {
    let credentials = InMemoryCredentialStore::default();
    let ollama = ProviderSettings::new("ollama", "http://localhost:11434", "qwen");
    assert!(ProviderFactory::new(&credentials).create(&ollama).is_ok());

    let remote = ProviderSettings::new("openai", "https://api.example.test", "model");
    let Err(error) = ProviderFactory::new(&credentials).create(&remote) else {
        panic!("remote provider must require a credential");
    };
    assert_eq!(error.code(), AppErrorCode::CredentialMissing);
}
