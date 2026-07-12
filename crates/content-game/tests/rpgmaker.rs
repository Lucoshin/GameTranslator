use std::path::PathBuf;

use game_translator_content_core::{
    ContentCategory, ContentSourceAdapter, OutputCapability, SegmentKind,
};
use game_translator_content_game::RpgMakerContentAdapter;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn rpg_maker_content_adapter_preserves_existing_extraction_results() {
    let adapter = RpgMakerContentAdapter;
    let source = adapter
        .detect(&fixture("rpgmaker-mz-dialogue"))
        .expect("fixture should be detected");
    let segments = adapter.extract(&source).expect("fixture should extract");

    assert_eq!(source.format_id, "game.rpgmaker.mz");
    assert_eq!(adapter.category(), ContentCategory::Game);
    assert_eq!(adapter.output_capabilities(), &[OutputCapability::Export]);
    let map_segment = segments
        .iter()
        .find(|segment| segment.source_file.ends_with("Map001.json"))
        .expect("fixture should include a map dialogue");
    assert_eq!(map_segment.source, "やっと着いた。 \\V[1]");
    assert_eq!(map_segment.kind, SegmentKind::Dialogue);
}
