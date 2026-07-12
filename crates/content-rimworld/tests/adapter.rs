use std::{collections::HashMap, fs, path::PathBuf};

use game_translator_content_core::{
    ContentCategory, ContentOutputAdapter, ContentSourceAdapter, ExportRequest, SegmentKind,
};
use game_translator_content_rimworld::{
    RimWorldLanguagePackOutputAdapter, RimWorldModContentAdapter,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rimworld-mod-minimal")
}

#[test]
fn detects_and_extracts_documented_rimworld_language_files() {
    let adapter = RimWorldModContentAdapter;
    let source = adapter
        .detect(&fixture())
        .expect("fixture should be detected");
    let segments = adapter.extract(&source).expect("fixture should extract");

    assert_eq!(source.format_id, "game.rimworld.mod");
    assert_eq!(source.source_id, "example.author.minimal");
    assert_eq!(adapter.category(), ContentCategory::GameMod);
    assert_eq!(
        adapter.output_capabilities(),
        &[game_translator_content_core::OutputCapability::Export]
    );
    assert_eq!(segments.len(), 3);

    let keyed = segments
        .iter()
        .find(|segment| segment.id == "rimworld:keyed:Keyed/Example.xml:ExampleGreeting")
        .expect("keyed entry should be extracted");
    assert_eq!(keyed.source, "Hello, {0}!");
    assert_eq!(keyed.kind, SegmentKind::LocalizedKey);
    assert_eq!(keyed.location, "ExampleGreeting");

    let definition = segments
        .iter()
        .find(|segment| {
            segment.id == "rimworld:definjected:DefInjected/ThingDef/Example.xml:ExampleThing.label"
        })
        .expect("DefInjected entry should be extracted");
    assert_eq!(definition.source, "Example thing");
}

#[test]
fn exports_translations_to_a_separate_rimworld_language_pack() {
    let source_adapter = RimWorldModContentAdapter;
    let source = source_adapter
        .detect(&fixture())
        .expect("fixture should be detected");
    let output = std::env::temp_dir().join(format!(
        "game-translator-rimworld-export-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&output);
    let translations = HashMap::from([(
        "rimworld:keyed:Keyed/Example.xml:ExampleGreeting".to_owned(),
        "你好，{0}！".to_owned(),
    )]);
    let adapter = RimWorldLanguagePackOutputAdapter;

    let result = adapter
        .export(&ExportRequest {
            source: &source,
            translations: &translations,
            output_root: &output,
            target_language: "zh-CN",
        })
        .expect("language pack should export");
    let keyed = result
        .output_root
        .join("Languages/ChineseSimplified/Keyed/Example.xml");

    assert_eq!(
        adapter.capabilities(),
        &[game_translator_content_core::OutputCapability::Export]
    );
    assert!(result
        .output_root
        .join("Languages/ChineseSimplified/LanguageInfo.xml")
        .is_file());
    assert!(keyed.is_file());
    assert!(fs::read_to_string(keyed).unwrap().contains("你好，{0}！"));
    let _ = fs::remove_dir_all(output);
}
