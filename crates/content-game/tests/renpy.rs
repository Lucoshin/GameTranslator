use std::path::PathBuf;

use game_translator_content_core::{
    ContentCategory, ContentError, ContentSourceAdapter, OutputCapability,
};
use game_translator_content_game::RenPyContentAdapter;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renpy-template")
}

#[test]
fn renpy_content_adapter_keeps_runtime_template_generation_as_a_requirement() {
    let adapter = RenPyContentAdapter;
    let source = adapter
        .detect(&fixture())
        .expect("fixture should be detected");
    let error = adapter
        .extract(&source)
        .expect_err("fixture has no runnable Ren'Py runtime");

    assert_eq!(source.format_id, "game.renpy");
    assert_eq!(adapter.category(), ContentCategory::Game);
    assert_eq!(
        adapter.output_capabilities(),
        &[
            OutputCapability::Export,
            OutputCapability::Install,
            OutputCapability::Uninstall
        ]
    );
    assert!(matches!(error, ContentError::Io { path, .. } if path.ends_with("Mayfly.exe")));
}
