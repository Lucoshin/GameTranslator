use std::path::Path;

use game_translator_engine_core::{DetectedProject, EngineError, EngineKind};

/// Detects a Ren'Py distribution without modifying it.
///
/// # Errors
/// Returns an engine error when required directories or a launcher are missing.
pub fn detect_project(root: &Path) -> Result<DetectedProject, EngineError> {
    let game_dir = root.join("game");
    if !root.join("renpy").is_dir() || !game_dir.is_dir() {
        return Err(EngineError::UnsupportedProject);
    }
    let has_launcher = std::fs::read_dir(root)
        .map_err(|error| EngineError::Io {
            path: root.to_path_buf(),
            message: error.to_string(),
        })?
        .filter_map(Result::ok)
        .any(|entry| {
            entry.path().extension().is_some_and(|extension| {
                extension.eq_ignore_ascii_case("exe") || extension.eq_ignore_ascii_case("py")
            })
        });
    if !has_launcher {
        return Err(EngineError::MissingRequiredFile(
            root.join("<game launcher>"),
        ));
    }
    Ok(DetectedProject {
        root: root.to_path_buf(),
        data_dir: game_dir,
        engine: EngineKind::RenPy,
    })
}
