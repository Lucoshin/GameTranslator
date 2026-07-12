use std::{collections::HashMap, hash::BuildHasher, path::Path};

use game_translator_content_core::{
    ContentError, ContentOutputAdapter, ContentSource, ContentSourceAdapter, ExportRequest,
    ExportResult, Segment,
};
use game_translator_content_game::{RenPyContentAdapter, RpgMakerContentAdapter};
use game_translator_content_rimworld::{
    RimWorldLanguagePackOutputAdapter, RimWorldModContentAdapter,
};

/// Detects a registered content source without modifying its files.
///
/// # Errors
///
/// Returns an error when no registered adapter recognizes the selected directory.
pub fn detect_content(root: &Path) -> Result<ContentSource, ContentError> {
    let adapters: [&dyn ContentSourceAdapter; 3] = [
        &RpgMakerContentAdapter,
        &RenPyContentAdapter,
        &RimWorldModContentAdapter,
    ];
    adapters
        .into_iter()
        .find_map(|adapter| adapter.detect(root).ok())
        .ok_or(ContentError::UnsupportedSource)
}

/// Extracts text segments using the adapter identified by a content source.
///
/// # Errors
///
/// Returns an error when the source format is not registered or cannot be read.
pub fn extract_content(source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
    match source.format_id {
        "game.rpgmaker.mv" | "game.rpgmaker.mz" => RpgMakerContentAdapter.extract(source),
        "game.renpy" => RenPyContentAdapter.extract(source),
        "game.rimworld.mod" => RimWorldModContentAdapter.extract(source),
        _ => Err(ContentError::UnsupportedSource),
    }
}

/// Exports content through the output adapter for its source format.
///
/// # Errors
///
/// Returns an error when the source format has no registered output adapter or output fails.
pub fn export_content<S: BuildHasher>(
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
