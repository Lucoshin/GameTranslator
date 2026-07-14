use std::{collections::HashMap, hash::BuildHasher, path::Path};

use game_translator_content_core::{
    ContentError, ContentOutputAdapter, ContentSource, ContentSourceAdapter, ExportRequest,
    ExportResult, Segment,
};
use game_translator_content_game::{RenPyContentAdapter, RpgMakerContentAdapter};
use game_translator_content_rimworld::{
    RimWorldLanguagePackOutputAdapter, RimWorldModContentAdapter,
};

const FORMAT_IDS: [&str; 4] = [
    "game.rpgmaker.mv",
    "game.rpgmaker.mz",
    "game.renpy",
    "game.rimworld.mod",
];

pub struct AdapterRegistry {
    source_adapters: [&'static dyn ContentSourceAdapter; 3],
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self {
            source_adapters: [
                &RpgMakerContentAdapter,
                &RenPyContentAdapter,
                &RimWorldModContentAdapter,
            ],
        }
    }
}

impl AdapterRegistry {
    #[must_use]
    pub const fn format_ids(&self) -> [&'static str; 4] {
        FORMAT_IDS
    }

    /// # Errors
    /// Returns [`ContentError::UnsupportedSource`] when no registered adapter accepts the root.
    pub fn detect(&self, root: &Path) -> Result<ContentSource, ContentError> {
        self.source_adapters
            .iter()
            .find_map(|adapter| adapter.detect(root).ok())
            .ok_or(ContentError::UnsupportedSource)
    }

    /// # Errors
    /// Returns an error when the source format is unknown or extraction fails.
    pub fn extract(&self, source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
        match source.format_id {
            "game.rpgmaker.mv" | "game.rpgmaker.mz" => RpgMakerContentAdapter.extract(source),
            "game.renpy" => RenPyContentAdapter.extract(source),
            "game.rimworld.mod" => RimWorldModContentAdapter.extract(source),
            _ => Err(ContentError::UnsupportedSource),
        }
    }

    /// # Errors
    /// Returns an error when the source has no registered standalone output adapter.
    pub fn export<S: BuildHasher>(
        &self,
        source: &ContentSource,
        translations: &HashMap<String, String, S>,
        output_root: &Path,
        target_language: &str,
    ) -> Result<ExportResult, ContentError> {
        match source.format_id {
            "game.rimworld.mod" => RimWorldLanguagePackOutputAdapter.export(&ExportRequest {
                source,
                translations,
                output_root,
                target_language,
            }),
            _ => Err(ContentError::UnsupportedSource),
        }
    }
}
