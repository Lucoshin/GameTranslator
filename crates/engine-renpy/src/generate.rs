use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use game_translator_engine_core::{DetectedProject, EngineError};

const PLUGIN: &str = r"init -999 python:
    import renpy.translation.generation as _gt_generation

    def _gt_all_translation_files():
        return list(renpy.game.script.translator.file_translates.keys())

    _gt_generation.translate_list_files = _gt_all_translation_files
";

/// Uses the bundled Ren'Py runtime to generate translation templates in a temporary directory.
///
/// # Errors
/// Returns an engine error when the launcher fails, templates cannot be copied, or cleanup fails.
pub fn generate_templates(project: &DetectedProject) -> Result<PathBuf, EngineError> {
    let token = format!("_gametranslator_{}_{}", std::process::id(), timestamp());
    let plugin = project.data_dir.join(format!("{token}.rpy"));
    let compiled_plugin = project.data_dir.join(format!("{token}.rpyc"));
    let generated = project.data_dir.join("tl").join(&token);
    let temporary = std::env::temp_dir().join(&token);
    if plugin.exists() || compiled_plugin.exists() || generated.exists() || temporary.exists() {
        return Err(EngineError::InvalidData {
            path: plugin,
            message: "temporary Ren'Py path already exists".into(),
        });
    }

    fs::write(&plugin, PLUGIN).map_err(|error| io_error(&plugin, error))?;
    let result = run_generator(project, &token)
        .and_then(|()| copy_tree(&generated, &temporary).map(|()| temporary.clone()));
    let cleanup_result = cleanup(&plugin, &compiled_plugin, &generated);
    match (result, cleanup_result) {
        (Ok(path), Ok(())) => Ok(path),
        (Err(error), _) | (Ok(_), Err(error)) => {
            let _ = fs::remove_dir_all(&temporary);
            Err(error)
        }
    }
}

fn run_generator(project: &DetectedProject, token: &str) -> Result<(), EngineError> {
    let launcher = find_launcher(&project.root)?;
    let output = Command::new(&launcher)
        .arg(&project.root)
        .args(["translate", token, "--empty"])
        .output()
        .map_err(|error| io_error(&launcher, error))?;
    if !output.status.success() {
        return Err(EngineError::InvalidData {
            path: launcher,
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    Ok(())
}

fn find_launcher(root: &Path) -> Result<PathBuf, EngineError> {
    let mut launchers = fs::read_dir(root)
        .map_err(|error| io_error(root, error))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        })
        .collect::<Vec<_>>();
    launchers.sort();
    launchers
        .into_iter()
        .next()
        .ok_or_else(|| EngineError::MissingRequiredFile(root.join("<game.exe>")))
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), EngineError> {
    fs::create_dir_all(target).map_err(|error| io_error(target, error))?;
    for entry in fs::read_dir(source).map_err(|error| io_error(source, error))? {
        let entry = entry.map_err(|error| io_error(source, error))?;
        let destination = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), &destination).map_err(|error| io_error(&destination, error))?;
        }
    }
    Ok(())
}

fn cleanup(plugin: &Path, compiled: &Path, generated: &Path) -> Result<(), EngineError> {
    for path in [plugin, compiled] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| io_error(path, error))?;
        }
    }
    if generated.exists() {
        fs::remove_dir_all(generated).map_err(|error| io_error(generated, error))?;
    }
    Ok(())
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn io_error(path: &Path, error: std::io::Error) -> EngineError {
    let message = error.to_string();
    drop(error);
    EngineError::Io {
        path: path.to_path_buf(),
        message,
    }
}
