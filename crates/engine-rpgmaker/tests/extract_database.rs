use std::path::PathBuf;

use game_translator_engine_core::SegmentKind;
use game_translator_engine_rpgmaker::{detect_project, extract_project};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rpgmaker-mz-dialogue")
}

#[test]
fn extracts_database_names_and_descriptions() {
    let project = detect_project(&fixture()).unwrap();
    let segments = extract_project(&project).unwrap();

    let potion = segments
        .iter()
        .find(|segment| segment.source == "回復薬")
        .unwrap();
    assert_eq!(potion.kind, SegmentKind::DatabaseName);
    assert_eq!(potion.id, "Items.json:records[1].name");

    let description = segments
        .iter()
        .find(|segment| segment.source == "HPを50回復する。")
        .unwrap();
    assert_eq!(description.kind, SegmentKind::DatabaseDescription);

    assert!(segments.iter().any(|segment| segment.source == "月光斬"));
}

#[test]
fn skips_blank_numeric_and_asset_path_values() {
    let project = detect_project(&fixture()).unwrap();
    let segments = extract_project(&project).unwrap();

    assert!(!segments
        .iter()
        .any(|segment| segment.source.trim().is_empty()));
    assert!(!segments.iter().any(|segment| segment.source == "12345"));
    assert!(!segments
        .iter()
        .any(|segment| segment.source == "img/icons/potion.png"));
}
