use std::{fs, path::Path};

use game_translator_engine_core::{
    DetectedProject, EngineError, Segment, SegmentContext, SegmentKind,
};
use serde_json::Value;

use crate::commands::{extract_command_list, is_translatable};

const DATABASE_FILES: &[&str] = &[
    "Actors.json",
    "Armors.json",
    "Classes.json",
    "Enemies.json",
    "Items.json",
    "Skills.json",
    "States.json",
    "Weapons.json",
];

/// Extracts supported translatable text from an RPG Maker project.
///
/// # Errors
///
/// Returns an [`EngineError`] when a discovered source file cannot be read or parsed.
pub fn extract_project(project: &DetectedProject) -> Result<Vec<Segment>, EngineError> {
    let mut files = fs::read_dir(&project.data_dir)
        .map_err(|error| EngineError::Io {
            path: project.data_dir.clone(),
            message: error.to_string(),
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    files.sort();

    let mut segments = Vec::new();
    for path in files {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name.starts_with("Map") && file_name != "MapInfos.json" {
            extract_map(&path, &mut segments)?;
        } else if file_name == "CommonEvents.json" {
            extract_common_events(&path, &mut segments)?;
        } else if DATABASE_FILES.contains(&file_name) {
            extract_database(&path, &mut segments)?;
        }
    }

    Ok(segments)
}

fn read_json(path: &Path) -> Result<Value, EngineError> {
    let content = fs::read_to_string(path).map_err(|error| EngineError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&content).map_err(|error| EngineError::InvalidData {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn extract_map(path: &Path, output: &mut Vec<Segment>) -> Result<(), EngineError> {
    let document = read_json(path)?;
    let Some(events) = document.get("events").and_then(Value::as_array) else {
        return Ok(());
    };

    for (event_index, event) in events.iter().enumerate() {
        let Some(pages) = event.get("pages").and_then(Value::as_array) else {
            continue;
        };
        for (page_index, page) in pages.iter().enumerate() {
            let Some(commands) = page.get("list").and_then(Value::as_array) else {
                continue;
            };
            output.extend(extract_command_list(
                commands,
                path,
                &format!("events[{event_index}].pages[{page_index}]"),
                &format!("events[{event_index}].pages[{page_index}]"),
            ));
        }
    }
    Ok(())
}

fn extract_common_events(path: &Path, output: &mut Vec<Segment>) -> Result<(), EngineError> {
    let document = read_json(path)?;
    let Some(records) = document.as_array() else {
        return Ok(());
    };

    for (record_index, record) in records.iter().enumerate() {
        let Some(commands) = record.get("list").and_then(Value::as_array) else {
            continue;
        };
        output.extend(extract_command_list(
            commands,
            path,
            &format!("records[{record_index}]"),
            &format!("[{record_index}]"),
        ));
    }
    Ok(())
}

fn extract_database(path: &Path, output: &mut Vec<Segment>) -> Result<(), EngineError> {
    let document = read_json(path)?;
    let Some(records) = document.as_array() else {
        return Ok(());
    };

    for (record_index, record) in records.iter().enumerate() {
        for (field, kind) in [
            ("name", SegmentKind::DatabaseName),
            ("description", SegmentKind::DatabaseDescription),
        ] {
            let Some(source) = record
                .get(field)
                .and_then(Value::as_str)
                .filter(|value| is_translatable(value))
            else {
                continue;
            };
            let id_path = format!("records[{record_index}].{field}");
            let json_path = format!("[{record_index}].{field}");
            let file_name = path
                .file_name()
                .expect("database file must have a file name")
                .to_string_lossy();
            output.push(Segment {
                id: format!("{file_name}:{id_path}"),
                source: source.to_owned(),
                source_file: path.to_path_buf(),
                location: json_path,
                kind,
                context: SegmentContext {
                    speaker: None,
                    previous_text: None,
                    next_text: None,
                },
            });
        }
    }
    Ok(())
}
