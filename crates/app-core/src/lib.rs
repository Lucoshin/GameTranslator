mod credentials;
mod patch;

pub use credentials::{
    CredentialError, CredentialStore, InMemoryCredentialStore, ProviderConfiguration,
    WindowsCredentialStore,
};
pub use patch::{PatchError, PatchFile, PatchManifest, PatchPlan};

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
