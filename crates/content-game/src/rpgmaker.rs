use std::path::Path;

use game_translator_content_core::{
    ContentCategory, ContentError, ContentSource, ContentSourceAdapter, OutputCapability, Segment,
};
use game_translator_engine_core::EngineKind;

use crate::error::map_engine_error;

pub struct RpgMakerContentAdapter;

impl ContentSourceAdapter for RpgMakerContentAdapter {
    fn format_id(&self) -> &'static str {
        "game.rpgmaker"
    }

    fn category(&self) -> ContentCategory {
        ContentCategory::Game
    }

    fn output_capabilities(&self) -> &'static [OutputCapability] {
        &[OutputCapability::Export]
    }

    fn detect(&self, root: &Path) -> Result<ContentSource, ContentError> {
        let project =
            game_translator_engine_rpgmaker::detect_project(root).map_err(map_engine_error)?;
        let (format_id, display_name, source_id) = match project.engine {
            EngineKind::RpgMakerMv => ("game.rpgmaker.mv", "RPG Maker MV project", "rpgmaker-mv"),
            EngineKind::RpgMakerMz => ("game.rpgmaker.mz", "RPG Maker MZ project", "rpgmaker-mz"),
            EngineKind::RenPy => return Err(ContentError::UnsupportedSource),
        };

        Ok(ContentSource {
            root: project.root,
            format_id,
            display_name: display_name.to_owned(),
            source_id: source_id.to_owned(),
        })
    }

    fn extract(&self, source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
        let project = game_translator_engine_rpgmaker::detect_project(&source.root)
            .map_err(map_engine_error)?;
        game_translator_engine_rpgmaker::extract_project(&project).map_err(map_engine_error)
    }
}
