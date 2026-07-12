use std::{
    collections::HashMap,
    fs,
    hash::BuildHasher,
    path::{Path, PathBuf},
};

use game_translator_engine_core::{DetectedProject, EngineError};

/// Writes standard Ren'Py translation files into an independent patch directory.
///
/// # Errors
/// Returns an engine error when templates cannot be generated, rewritten, or saved.
pub fn write_translations<S: BuildHasher>(
    project: &DetectedProject,
    translations: &HashMap<String, String, S>,
    output_root: &Path,
    language: &str,
) -> Result<Vec<PathBuf>, EngineError> {
    let templates = crate::generate_templates(project)?;
    let target_root = output_root.join("game/tl").join(language);
    let result = rewrite_tree(&templates, &templates, &target_root, translations, language);
    let _ = fs::remove_dir_all(templates);
    result
}

fn rewrite_tree<S: BuildHasher>(
    root: &Path,
    current: &Path,
    target: &Path,
    translations: &HashMap<String, String, S>,
    language: &str,
) -> Result<Vec<PathBuf>, EngineError> {
    fs::create_dir_all(target).map_err(|error| io_error(target, error))?;
    let mut written = Vec::new();
    for entry in fs::read_dir(current).map_err(|error| io_error(current, error))? {
        let entry = entry.map_err(|error| io_error(current, error))?;
        let destination = target.join(entry.file_name());
        if entry.path().is_dir() {
            written.extend(rewrite_tree(
                root,
                &entry.path(),
                &destination,
                translations,
                language,
            )?);
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rpy"))
        {
            let rendered = rewrite_file(root, &entry.path(), translations, language)?;
            fs::write(&destination, rendered).map_err(|error| io_error(&destination, error))?;
            written.push(destination);
        }
    }
    Ok(written)
}

fn rewrite_file<S: BuildHasher>(
    root: &Path,
    path: &Path,
    translations: &HashMap<String, String, S>,
    language: &str,
) -> Result<String, EngineError> {
    let content = fs::read_to_string(path).map_err(|error| io_error(path, error))?;
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut lines = content.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut block = String::new();
    for index in 0..lines.len() {
        let trimmed = lines[index].trim().to_owned();
        if let Some(header) = trimmed.strip_prefix("translate ") {
            header
                .trim_end_matches(':')
                .split_once(' ')
                .map_or(header, |(_, id)| id)
                .clone_into(&mut block);
            lines[index] = lines[index].replacen(
                header.split_whitespace().next().unwrap_or_default(),
                language,
                1,
            );
        }
        let is_source = trimmed.starts_with("# ") || trimmed.starts_with("old ");
        if is_source {
            let id = format!("renpy:{}::{block}::{}", relative.display(), index + 1);
            if let Some(translation) = translations.get(&id) {
                if let Some(target_index) = ((index + 1)..lines.len()).find(|candidate| {
                    let value = lines[*candidate].trim();
                    !value.is_empty() && !value.starts_with('#')
                }) {
                    lines[target_index] = replace_quoted(&lines[target_index], translation);
                }
            }
        }
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn replace_quoted(line: &str, translation: &str) -> String {
    let Some(start) = line.find('"') else {
        return line.to_owned();
    };
    let Some(end) = line.rfind('"') else {
        return line.to_owned();
    };
    let escaped = translation
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("{}\"{}\"{}", &line[..start], escaped, &line[end + 1..])
}

fn io_error(path: &Path, error: std::io::Error) -> EngineError {
    let message = error.to_string();
    drop(error);
    EngineError::Io {
        path: path.to_path_buf(),
        message,
    }
}
