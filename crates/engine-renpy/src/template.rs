use std::{fs, path::Path};

use game_translator_engine_core::{
    DetectedProject, EngineError, Segment, SegmentContext, SegmentKind,
};

/// Extracts segments from a checked-in or pre-generated test template directory.
///
/// # Errors
/// Returns an engine error when template files cannot be read or parsed.
pub fn extract_templates(project: &DetectedProject) -> Result<Vec<Segment>, EngineError> {
    let template_root = project.data_dir.join("tl/_gametranslator_template");
    extract_from(project, &template_root)
}

/// Generates templates with the bundled runtime and extracts stable translation segments.
///
/// # Errors
/// Returns an engine error when generation, reading, or cleanup fails.
pub fn extract_project(project: &DetectedProject) -> Result<Vec<Segment>, EngineError> {
    let template_root = crate::generate_templates(project)?;
    extract_from(project, &template_root)
}

fn extract_from(
    project: &DetectedProject,
    template_root: &Path,
) -> Result<Vec<Segment>, EngineError> {
    let mut files = Vec::new();
    collect_rpy_files(template_root, &mut files)?;
    files.sort();
    let mut segments = Vec::new();
    for path in files {
        parse_template(project, template_root, &path, &mut segments)?;
    }
    Ok(segments)
}

fn collect_rpy_files(root: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), EngineError> {
    for entry in fs::read_dir(root).map_err(|error| EngineError::Io {
        path: root.to_path_buf(),
        message: error.to_string(),
    })? {
        let entry = entry.map_err(|error| EngineError::Io {
            path: root.to_path_buf(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_rpy_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rpy"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_template(
    project: &DetectedProject,
    template_root: &Path,
    path: &Path,
    segments: &mut Vec<Segment>,
) -> Result<(), EngineError> {
    let content = fs::read_to_string(path).map_err(|error| EngineError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let lines = content.lines().collect::<Vec<_>>();
    let source_file = primary_source_file(project);
    let mut block = String::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(header) = trimmed.strip_prefix("translate ") {
            header
                .trim_end_matches(':')
                .split_once(' ')
                .map_or(header, |(_, id)| id)
                .clone_into(&mut block);
        }
        if let Some(comment) = trimmed.strip_prefix("# ") {
            if let Some(source) = quoted_text(comment) {
                let speaker = comment.split_whitespace().next().map(str::to_owned);
                segments.push(segment(
                    &source_file,
                    template_root,
                    path,
                    &block,
                    index + 1,
                    source,
                    (speaker, SegmentKind::Dialogue),
                ));
            }
        } else if let Some(old) = trimmed.strip_prefix("old ") {
            if let Some(source) = quoted_text(old) {
                segments.push(segment(
                    &source_file,
                    template_root,
                    path,
                    &block,
                    index + 1,
                    source,
                    (None, SegmentKind::Choice),
                ));
            }
        }
    }
    Ok(())
}

fn segment(
    source_file: &Path,
    template_root: &Path,
    path: &Path,
    block: &str,
    line: usize,
    source: String,
    metadata: (Option<String>, SegmentKind),
) -> Segment {
    let relative = path.strip_prefix(template_root).unwrap_or(path);
    Segment {
        id: format!("renpy:{}::{block}::{line}", relative.display()),
        source,
        source_file: source_file.to_path_buf(),
        location: format!("{}::{block}::{line}", relative.display()),
        kind: metadata.1,
        context: SegmentContext {
            speaker: metadata.0,
            previous_text: None,
            next_text: None,
        },
    }
}

fn primary_source_file(project: &DetectedProject) -> std::path::PathBuf {
    fs::read_dir(&project.data_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("rpa"))
        })
        .unwrap_or_else(|| project.data_dir.clone())
}

fn quoted_text(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    (end > start).then(|| {
        line[start + 1..end]
            .replace("\\\"", "\"")
            .replace("\\n", "\n")
    })
}
