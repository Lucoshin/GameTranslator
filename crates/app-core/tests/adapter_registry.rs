use std::path::PathBuf;

use game_translator_app_core::AdapterRegistry;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn one_registry_owns_all_supported_content_formats() {
    let registry = AdapterRegistry::default();
    assert_eq!(
        registry.format_ids(),
        [
            "game.rpgmaker.mv",
            "game.rpgmaker.mz",
            "game.renpy",
            "game.rimworld.mod"
        ]
    );
    assert_eq!(
        registry
            .detect(&fixture("rpgmaker-mz-minimal"))
            .unwrap()
            .format_id,
        "game.rpgmaker.mz"
    );
    assert_eq!(
        registry
            .detect(&fixture("renpy-template"))
            .unwrap()
            .format_id,
        "game.renpy"
    );
    assert_eq!(
        registry
            .detect(&fixture("rimworld-mod-minimal"))
            .unwrap()
            .format_id,
        "game.rimworld.mod"
    );
}
