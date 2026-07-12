use std::path::{Path, PathBuf};

use game_translator_content_core::{
    ContentCategory, ContentError, ContentSource, ContentSourceAdapter, OutputCapability, Segment,
};

struct StaticSourceAdapter;

impl ContentSourceAdapter for StaticSourceAdapter {
    fn format_id(&self) -> &'static str {
        "test.static"
    }

    fn category(&self) -> ContentCategory {
        ContentCategory::Document
    }

    fn output_capabilities(&self) -> &'static [OutputCapability] {
        &[OutputCapability::Export]
    }

    fn detect(&self, root: &Path) -> Result<ContentSource, ContentError> {
        Ok(ContentSource {
            root: root.to_path_buf(),
            format_id: self.format_id(),
            display_name: "Static document".to_owned(),
            source_id: "test-static".to_owned(),
        })
    }

    fn extract(&self, _source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
        Ok(Vec::new())
    }
}

#[test]
fn source_adapter_declares_identity_category_and_safe_output_capabilities() {
    let adapter = StaticSourceAdapter;
    let root = PathBuf::from("fixture");

    let source = adapter.detect(&root).expect("source should be detected");

    assert_eq!(adapter.format_id(), "test.static");
    assert_eq!(adapter.category(), ContentCategory::Document);
    assert_eq!(adapter.output_capabilities(), &[OutputCapability::Export]);
    assert_eq!(source.root, root);
    assert_eq!(source.format_id, "test.static");
}
