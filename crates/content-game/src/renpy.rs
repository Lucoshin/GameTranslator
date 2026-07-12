use std::path::Path;

use game_translator_content_core::{
    ContentCategory, ContentError, ContentSource, ContentSourceAdapter, OutputCapability, Segment,
};

use crate::error::map_engine_error;

pub struct RenPyContentAdapter;

impl ContentSourceAdapter for RenPyContentAdapter {
    fn format_id(&self) -> &'static str {
        "game.renpy"
    }

    fn category(&self) -> ContentCategory {
        ContentCategory::Game
    }

    fn output_capabilities(&self) -> &'static [OutputCapability] {
        &[
            OutputCapability::Export,
            OutputCapability::Install,
            OutputCapability::Uninstall,
        ]
    }

    fn detect(&self, root: &Path) -> Result<ContentSource, ContentError> {
        let project =
            game_translator_engine_renpy::detect_project(root).map_err(map_engine_error)?;

        Ok(ContentSource {
            root: project.root,
            format_id: self.format_id(),
            display_name: "Ren'Py project".to_owned(),
            source_id: "renpy".to_owned(),
        })
    }

    fn extract(&self, source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
        let project =
            game_translator_engine_renpy::detect_project(&source.root).map_err(map_engine_error)?;
        game_translator_engine_renpy::extract_project(&project).map_err(map_engine_error)
    }
}
