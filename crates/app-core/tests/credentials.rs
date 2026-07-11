use game_translator_app_core::{CredentialStore, InMemoryCredentialStore, ProviderConfiguration};

#[test]
fn credential_store_supports_write_read_and_delete() {
    let mut store = InMemoryCredentialStore::default();

    store.set("provider-1", "secret-key").unwrap();
    assert_eq!(
        store.get("provider-1").unwrap().as_deref(),
        Some("secret-key")
    );
    store.delete("provider-1").unwrap();
    assert_eq!(store.get("provider-1").unwrap(), None);
}

#[test]
fn serializable_provider_configuration_never_contains_an_api_key() {
    let configuration = ProviderConfiguration {
        id: "provider-1".into(),
        base_url: "https://api.example.com/v1".into(),
        model: "deepseek-chat".into(),
    };

    let serialized = serde_json::to_string(&configuration).unwrap();

    assert!(!serialized.to_ascii_lowercase().contains("api_key"));
    assert!(!serialized.contains("secret-key"));
}
