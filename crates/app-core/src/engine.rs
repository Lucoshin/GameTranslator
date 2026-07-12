use std::path::Path;

use game_translator_engine_core::{
    DetectedProject, EngineAdapter, EngineError, EngineKind, Segment,
};
use game_translator_engine_renpy::RenPyAdapter;
use game_translator_engine_rpgmaker::RpgMakerAdapter;

/// Detects a game using the registered engine adapters.
///
/// # Errors
/// Returns an engine error when no adapter recognizes the directory.
pub fn detect_game(root: &Path) -> Result<DetectedProject, EngineError> {
    let adapters: [&dyn EngineAdapter; 2] = [&RpgMakerAdapter, &RenPyAdapter];
    adapters
        .into_iter()
        .find_map(|adapter| adapter.detect(root).ok())
        .ok_or(EngineError::UnsupportedProject)
}

/// Extracts translatable segments with the adapter selected during detection.
///
/// # Errors
/// Returns an engine error when engine data cannot be read or templates cannot be generated.
pub fn extract_game(project: &DetectedProject) -> Result<Vec<Segment>, EngineError> {
    match project.engine {
        EngineKind::RpgMakerMv | EngineKind::RpgMakerMz => RpgMakerAdapter.extract(project),
        EngineKind::RenPy => RenPyAdapter.extract(project),
    }
}

#[must_use]
pub const fn engine_name(engine: EngineKind) -> &'static str {
    match engine {
        EngineKind::RpgMakerMv => "RPG Maker MV",
        EngineKind::RpgMakerMz => "RPG Maker MZ",
        EngineKind::RenPy => "Ren'Py",
    }
}
