use std::path::PathBuf;

use game_translator_engine_core::SegmentKind;
use game_translator_engine_rpgmaker::{detect_project, extract_project};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rpgmaker-mz-dialogue")
}

#[test]
fn extracts_map_dialogue_choices_and_scrolling_text_with_context() {
    let project = detect_project(&fixture()).unwrap();
    let segments = extract_project(&project).unwrap();
    let map_segments: Vec<_> = segments
        .iter()
        .filter(|segment| segment.source_file.ends_with("Map001.json"))
        .collect();

    assert_eq!(map_segments.len(), 5);
    assert_eq!(map_segments[0].source, "やっと着いた。 \\V[1]");
    assert_eq!(map_segments[0].kind, SegmentKind::Dialogue);
    assert_eq!(map_segments[0].context.speaker.as_deref(), Some("アリス"));
    assert_eq!(
        map_segments[0].context.next_text.as_deref(),
        Some("ここが月の神殿ね。")
    );
    assert_eq!(map_segments[0].id, "Map001.json:events[1].pages[0].list[1]");
    assert_eq!(
        map_segments[0].location,
        "events[1].pages[0].list[1].parameters[0]"
    );
    assert_eq!(map_segments[2].kind, SegmentKind::Choice);
    assert_eq!(map_segments[2].source, "中に入る");
    assert_eq!(map_segments[4].kind, SegmentKind::ScrollingText);
}

#[test]
fn extracts_common_event_dialogue_and_skips_scripts() {
    let project = detect_project(&fixture()).unwrap();
    let segments = extract_project(&project).unwrap();
    let common: Vec<_> = segments
        .iter()
        .filter(|segment| segment.source_file.ends_with("CommonEvents.json"))
        .collect();

    assert_eq!(common.len(), 1);
    assert_eq!(common[0].source, "共通イベントの台詞");
    assert!(!segments
        .iter()
        .any(|segment| segment.source.contains("$game")));
}
