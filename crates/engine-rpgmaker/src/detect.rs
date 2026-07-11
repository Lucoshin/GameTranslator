use std::{fs, path::Path};

use game_translator_engine_core::{DetectedProject, EngineError, EngineKind};

/// Detects an unencrypted RPG Maker MV or MZ project directory.
///
/// # Errors
///
/// Returns [`EngineError::UnsupportedProject`] when no supported data directory exists, or a
/// more specific error when the system database is missing, unreadable, or malformed.
pub fn detect_project(root: &Path) -> Result<DetectedProject, EngineError> {
    let (engine, data_dir) = if root.join("www/data").is_dir() {
        (EngineKind::RpgMakerMv, root.join("www/data"))
    } else if root.join("data").is_dir() {
        (EngineKind::RpgMakerMz, root.join("data"))
    } else {
        return Err(EngineError::UnsupportedProject);
    };

    let system_path = data_dir.join("System.json");
    if !system_path.is_file() {
        return Err(EngineError::MissingRequiredFile(system_path));
    }

    let system = fs::read_to_string(&system_path).map_err(|error| EngineError::Io {
        path: system_path.clone(),
        message: error.to_string(),
    })?;
    serde_json::from_str::<serde_json::Value>(&system).map_err(|error| {
        EngineError::InvalidData {
            path: system_path,
            message: error.to_string(),
        }
    })?;

    Ok(DetectedProject {
        root: root.to_path_buf(),
        data_dir,
        engine,
    })
}
