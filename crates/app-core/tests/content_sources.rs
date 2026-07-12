use std::path::PathBuf;

use game_translator_app_core::{detect_content, extract_content};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn detects_and_extracts_a_registered_content_source() {
    let source = detect_content(&fixture("rpgmaker-mz-dialogue"))
        .expect("registered RPG Maker adapter should detect the fixture");
    let segments = extract_content(&source).expect("registered adapter should extract the fixture");

    assert_eq!(source.format_id, "game.rpgmaker.mz");
    assert!(!segments.is_empty());
}

#[test]
fn detects_and_extracts_a_registered_rimworld_mod_source() {
    let source = detect_content(&fixture("rimworld-mod-minimal"))
        .expect("registered RimWorld adapter should detect the fixture");
    let segments = extract_content(&source).expect("registered adapter should extract the fixture");

    assert_eq!(source.format_id, "game.rimworld.mod");
    assert_eq!(source.source_id, "example.author.minimal");
    assert_eq!(segments.len(), 3);
}
