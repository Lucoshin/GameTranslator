use std::{collections::HashMap, fs, hash::BuildHasher, path::Path, path::PathBuf};

use game_translator_engine_core::{DetectedProject, EngineError};
use serde_json::Value;

use crate::extract_project;

/// Writes translated strings into a separate project-shaped directory.
///
/// # Errors
///
/// Returns [`EngineError`] when source JSON cannot be read, a recorded path no longer resolves,
/// output directories cannot be created, or rendered JSON cannot be written.
pub fn write_translations<S: BuildHasher>(
    project: &DetectedProject,
    translations: &HashMap<String, String, S>,
    output_root: &Path,
) -> Result<Vec<PathBuf>, EngineError> {
    let segments = extract_project(project)?;
    let mut by_file: HashMap<PathBuf, Vec<_>> = HashMap::new();
    for segment in &segments {
        if let Some(translation) = translations.get(&segment.id) {
            by_file
                .entry(segment.source_file.clone())
                .or_default()
                .push((segment, translation));
        }
    }

    let mut source_files = by_file.into_iter().collect::<Vec<_>>();
    source_files.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut written = Vec::with_capacity(source_files.len());
    for (source_path, replacements) in source_files {
        let content = fs::read_to_string(&source_path).map_err(|error| EngineError::Io {
            path: source_path.clone(),
            message: error.to_string(),
        })?;
        let mut document: Value =
            serde_json::from_str(&content).map_err(|error| EngineError::InvalidData {
                path: source_path.clone(),
                message: error.to_string(),
            })?;
        for (segment, translation) in replacements {
            let target = value_at_path_mut(&mut document, &segment.json_path).ok_or_else(|| {
                EngineError::InvalidData {
                    path: source_path.clone(),
                    message: format!("translation path no longer exists: {}", segment.json_path),
                }
            })?;
            if !target.is_string() {
                return Err(EngineError::InvalidData {
                    path: source_path.clone(),
                    message: format!("translation path is not a string: {}", segment.json_path),
                });
            }
            *target = Value::String(translation.clone());
        }

        let relative =
            source_path
                .strip_prefix(&project.root)
                .map_err(|error| EngineError::InvalidData {
                    path: source_path.clone(),
                    message: error.to_string(),
                })?;
        let output_path = output_root.join(relative);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| EngineError::Io {
                path: parent.to_path_buf(),
                message: error.to_string(),
            })?;
        }
        let rendered =
            serde_json::to_vec_pretty(&document).map_err(|error| EngineError::InvalidData {
                path: source_path.clone(),
                message: error.to_string(),
            })?;
        fs::write(&output_path, rendered).map_err(|error| EngineError::Io {
            path: output_path.clone(),
            message: error.to_string(),
        })?;
        written.push(output_path);
    }
    Ok(written)
}

fn value_at_path_mut<'a>(document: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    let mut current = document;
    for component in path.split('.') {
        let field_end = component.find('[').unwrap_or(component.len());
        let field = &component[..field_end];
        if !field.is_empty() {
            current = current.get_mut(field)?;
        }

        let mut remainder = &component[field_end..];
        while let Some(index_text) = remainder.strip_prefix('[') {
            let closing = index_text.find(']')?;
            let index = index_text[..closing].parse::<usize>().ok()?;
            current = current.get_mut(index)?;
            remainder = &index_text[closing + 1..];
        }
        if !remainder.is_empty() {
            return None;
        }
    }
    Some(current)
}
