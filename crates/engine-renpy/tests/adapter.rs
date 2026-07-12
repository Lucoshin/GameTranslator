use std::path::PathBuf;

use game_translator_engine_core::{EngineKind, SegmentKind};
use game_translator_engine_renpy::{detect_project, extract_templates};

#[test]
fn detects_a_renpy_distribution() {
    let root = fixture();

    let project = detect_project(&root).unwrap();

    assert_eq!(project.engine, EngineKind::RenPy);
    assert_eq!(project.data_dir, root.join("game"));
}

#[test]
fn extracts_dialogue_and_string_translation_blocks() {
    let root = fixture();
    let project = detect_project(&root).unwrap();

    let segments = extract_templates(&project).unwrap();

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].source, "欢迎光临~");
    assert_eq!(segments[0].context.speaker.as_deref(), Some("ji"));
    assert_eq!(segments[0].kind, SegmentKind::Dialogue);
    assert_eq!(segments[1].source, "Start");
    assert_eq!(segments[1].kind, SegmentKind::Choice);
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renpy-template")
}
