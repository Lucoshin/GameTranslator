use std::path::Path;

use game_translator_engine_core::{Segment, SegmentContext, SegmentKind};
use serde_json::Value;

pub(crate) fn extract_command_list(
    commands: &[Value],
    source_file: &Path,
    id_prefix: &str,
    json_prefix: &str,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut speaker = None;

    for (command_index, command) in commands.iter().enumerate() {
        let Some(code) = command.get("code").and_then(Value::as_i64) else {
            continue;
        };
        let parameters = command.get("parameters").and_then(Value::as_array);

        match code {
            101 => {
                speaker = parameters
                    .and_then(|values| values.get(4))
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_owned);
            }
            401 => push_parameter_segment(
                &mut segments,
                parameters,
                source_file,
                &format!("{id_prefix}.list[{command_index}]"),
                &format!("{json_prefix}.list[{command_index}]"),
                SegmentKind::Dialogue,
                speaker.clone(),
            ),
            102 => {
                if let Some(choices) = parameters
                    .and_then(|values| values.first())
                    .and_then(Value::as_array)
                {
                    for (choice_index, choice) in choices.iter().enumerate() {
                        if let Some(text) = choice.as_str().filter(|value| is_translatable(value)) {
                            let id_path = format!(
                                "{id_prefix}.list[{command_index}].parameters[0][{choice_index}]"
                            );
                            let json_path = format!(
                                "{json_prefix}.list[{command_index}].parameters[0][{choice_index}]"
                            );
                            segments.push(new_segment(
                                source_file,
                                &id_path,
                                json_path.clone(),
                                text,
                                SegmentKind::Choice,
                                speaker.clone(),
                            ));
                        }
                    }
                }
            }
            405 => push_parameter_segment(
                &mut segments,
                parameters,
                source_file,
                &format!("{id_prefix}.list[{command_index}]"),
                &format!("{json_prefix}.list[{command_index}]"),
                SegmentKind::ScrollingText,
                speaker.clone(),
            ),
            _ => {}
        }
    }

    attach_neighbor_context(&mut segments);
    segments
}

fn push_parameter_segment(
    segments: &mut Vec<Segment>,
    parameters: Option<&Vec<Value>>,
    source_file: &Path,
    id_command_path: &str,
    json_command_path: &str,
    kind: SegmentKind,
    speaker: Option<String>,
) {
    let Some(text) = parameters
        .and_then(|values| values.first())
        .and_then(Value::as_str)
        .filter(|value| is_translatable(value))
    else {
        return;
    };

    segments.push(new_segment(
        source_file,
        id_command_path,
        format!("{json_command_path}.parameters[0]"),
        text,
        kind,
        speaker,
    ));
}

fn new_segment(
    source_file: &Path,
    id_path: &str,
    json_path: String,
    source: &str,
    kind: SegmentKind,
    speaker: Option<String>,
) -> Segment {
    let file_name = source_file
        .file_name()
        .expect("source file must have a file name")
        .to_string_lossy();

    Segment {
        id: format!("{file_name}:{id_path}"),
        source: source.to_owned(),
        source_file: source_file.to_path_buf(),
        json_path,
        kind,
        context: SegmentContext {
            speaker,
            previous_text: None,
            next_text: None,
        },
    }
}

fn attach_neighbor_context(segments: &mut [Segment]) {
    for index in 0..segments.len() {
        if index > 0 {
            segments[index].context.previous_text = Some(segments[index - 1].source.clone());
        }
        if index + 1 < segments.len() {
            segments[index].context.next_text = Some(segments[index + 1].source.clone());
        }
    }
}

pub(crate) fn is_translatable(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.chars().all(|character| character.is_ascii_digit())
}
