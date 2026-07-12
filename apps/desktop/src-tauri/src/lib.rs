use std::{collections::HashMap, path::Path, path::PathBuf};

use game_translator_app_core::{
    detect_game, engine_name, extract_game, CredentialStore, PatchPlan, WindowsCredentialStore,
};
use game_translator_engine_core::Segment;
use game_translator_provider_core::{
    OllamaProvider, OpenAiCompatibleProvider, TranslationProvider,
};
use game_translator_qa_core::{
    check_translation, protect_placeholders, restore_placeholders, validate_control_codes,
    ProtectedText, QaCode, QaFinding, QaSeverity,
};
use game_translator_translation_core::{
    RunControl, RunResult, TranslationOrchestrator, TranslationSegment,
};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    project_path: String,
    project_name: String,
    engine: String,
    segment_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderInput {
    kind: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslateCommandInput {
    run_id: String,
    project_path: String,
    provider: ProviderInput,
    source_language: LanguageInput,
    target_language: LanguageInput,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationProgressEvent {
    run_id: String,
    phase: String,
    completed: usize,
    total: usize,
    failed: usize,
    warning_findings: usize,
    blocking_findings: usize,
    message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LanguageInput {
    code: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationItem {
    id: String,
    source: String,
    target: String,
    speaker: Option<String>,
    source_file: String,
    qa: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationRunResult {
    items: Vec<TranslationItem>,
    warning_findings: usize,
    blocking_findings: usize,
    failed_segment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportCommandInput {
    project_path: String,
    items: Vec<TranslationItem>,
    target_language: LanguageInput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportResult {
    output_path: String,
    file_count: usize,
}

#[tauri::command]
fn select_and_scan_project() -> Result<ScanResult, String> {
    let path = rfd::FileDialog::new()
        .set_title("选择 RPG Maker 游戏目录")
        .pick_folder()
        .ok_or_else(|| "未选择游戏目录".to_owned())?;
    scan_path(&path)
}

fn scan_path(path: &Path) -> Result<ScanResult, String> {
    let project = detect_game(path).map_err(|error| error.to_string())?;
    let segments = extract_game(&project).map_err(|error| error.to_string())?;
    let engine = engine_name(project.engine);
    Ok(ScanResult {
        project_path: path.to_string_lossy().into_owned(),
        project_name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("未命名项目")
            .to_owned(),
        engine: engine.to_owned(),
        segment_count: segments.len(),
    })
}

#[tauri::command]
fn save_provider_configuration(provider: ProviderInput) -> Result<(), String> {
    let ProviderInput {
        kind,
        api_key,
        base_url: _,
        model: _,
    } = provider;
    if kind == "openai" {
        let secret = api_key
            .as_deref()
            .ok_or_else(|| "OpenAI-compatible Provider 需要 API Key".to_owned())?;
        let mut credentials = WindowsCredentialStore::new("GameTranslator");
        credentials
            .set("default-provider", secret)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn translate_project(
    app: tauri::AppHandle,
    input: TranslateCommandInput,
) -> Result<TranslationRunResult, String> {
    tauri::async_runtime::spawn_blocking(move || translate_command(&app, input))
        .await
        .map_err(|error| format!("翻译任务异常终止: {error}"))?
}

fn translate_command(
    app: &tauri::AppHandle,
    input: TranslateCommandInput,
) -> Result<TranslationRunResult, String> {
    let TranslateCommandInput {
        run_id,
        project_path,
        source_language,
        target_language,
        provider:
            ProviderInput {
                kind,
                base_url,
                model,
                api_key: _,
            },
    } = input;
    emit_progress(
        app,
        &run_id,
        "extracting",
        0,
        0,
        0,
        0,
        0,
        "正在提取游戏文本",
    );
    let provider: Box<dyn TranslationProvider> = if kind == "ollama" {
        Box::new(OllamaProvider::new(&base_url))
    } else {
        let credentials = WindowsCredentialStore::new("GameTranslator");
        let api_key = credentials
            .get("default-provider")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "尚未保存 API Key".to_owned())?;
        Box::new(OpenAiCompatibleProvider::new(&base_url, api_key))
    };
    translate_path_with_provider_and_progress(
        Path::new(&project_path),
        provider.as_ref(),
        &model,
        &language_prompt(&source_language),
        &language_prompt(&target_language),
        |phase, completed, total, failed, warnings, blocking, message| {
            emit_progress(
                app, &run_id, phase, completed, total, failed, warnings, blocking, message,
            );
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_progress(
    app: &tauri::AppHandle,
    run_id: &str,
    phase: &str,
    completed: usize,
    total: usize,
    failed: usize,
    warning_findings: usize,
    blocking_findings: usize,
    message: &str,
) {
    let _ = app.emit(
        "translation-progress",
        TranslationProgressEvent {
            run_id: run_id.to_owned(),
            phase: phase.to_owned(),
            completed,
            total,
            failed,
            warning_findings,
            blocking_findings,
            message: message.to_owned(),
        },
    );
}

fn language_prompt(language: &LanguageInput) -> String {
    if language.code == "auto" {
        "Auto-detect the source language".to_owned()
    } else {
        format!("{} ({})", language.name, language.code)
    }
}

#[cfg(test)]
fn translate_path_with_provider(
    path: &Path,
    provider: &dyn TranslationProvider,
    model: &str,
    source_language: &str,
    target_language: &str,
) -> Result<TranslationRunResult, String> {
    translate_path_with_provider_and_progress(
        path,
        provider,
        model,
        source_language,
        target_language,
        |_, _, _, _, _, _, _| {},
    )
}

fn translate_path_with_provider_and_progress<F>(
    path: &Path,
    provider: &dyn TranslationProvider,
    model: &str,
    source_language: &str,
    target_language: &str,
    mut progress: F,
) -> Result<TranslationRunResult, String>
where
    F: FnMut(&str, usize, usize, usize, usize, usize, &str),
{
    let project = detect_game(path).map_err(|error| error.to_string())?;
    let segments = extract_game(&project).map_err(|error| error.to_string())?;
    let total = segments.len();
    progress(
        "translating",
        0,
        total,
        0,
        0,
        0,
        "文本提取完成，正在请求模型",
    );
    let mut protected = HashMap::new();
    let translation_segments = segments
        .iter()
        .map(|segment| {
            let protected_text = protect_placeholders(&segment.source);
            let source = protected_text.text.clone();
            protected.insert(segment.id.clone(), protected_text);
            TranslationSegment::new(&segment.id, segment.source_file.to_string_lossy(), source)
        })
        .collect::<Vec<_>>();
    let orchestrator =
        TranslationOrchestrator::new(provider, model, source_language, target_language, 32);
    let run = orchestrator.run_with_progress(
        &translation_segments,
        &HashMap::new(),
        RunControl::Running,
        |current| {
            let completed = current.translations.len() + current.failed_segment_ids.len();
            progress(
                "translating",
                completed,
                total,
                current.failed_segment_ids.len(),
                0,
                0,
                &format!("模型已处理 {completed} / {total} 个片段"),
            );
        },
    );
    progress(
        "qa",
        0,
        total,
        run.failed_segment_ids.len(),
        0,
        0,
        "模型翻译结束，正在执行质量检查",
    );
    let (items, warning_findings, blocking_findings) =
        restore_and_check(segments, &run, &protected, &mut progress)?;

    progress(
        "completed",
        total,
        total,
        run.failed_segment_ids.len(),
        warning_findings,
        blocking_findings,
        "任务完成，可以校对或导出",
    );

    Ok(TranslationRunResult {
        items,
        warning_findings,
        blocking_findings,
        failed_segment_ids: run.failed_segment_ids,
    })
}

fn restore_and_check<F>(
    segments: Vec<Segment>,
    run: &RunResult,
    protected: &HashMap<String, ProtectedText>,
    progress: &mut F,
) -> Result<(Vec<TranslationItem>, usize, usize), String>
where
    F: FnMut(&str, usize, usize, usize, usize, usize, &str),
{
    let total = segments.len();
    let mut items = Vec::with_capacity(run.translations.len());
    let mut warning_findings = 0;
    let mut blocking_findings = 0;
    for (index, segment) in segments.into_iter().enumerate() {
        let Some(translated) = run.translations.get(&segment.id) else {
            continue;
        };
        let target = restore_placeholders(
            protected
                .get(&segment.id)
                .ok_or_else(|| format!("缺少占位符映射: {}", segment.id))?,
            translated,
        )
        .map_err(|error| format!("{}: {error}", segment.id))?;
        let findings = check_translation(&segment.source, &target, None);
        let mut item_qa = "passed";
        for finding in findings {
            match finding.severity {
                QaSeverity::Blocking => {
                    blocking_findings += 1;
                    item_qa = "blocking";
                }
                QaSeverity::Warning => {
                    warning_findings += 1;
                    if item_qa == "passed" {
                        item_qa = "warning";
                    }
                }
            }
        }
        items.push(TranslationItem {
            id: segment.id,
            source: segment.source,
            target,
            speaker: segment.context.speaker,
            source_file: segment.source_file.to_string_lossy().into_owned(),
            qa: item_qa.to_owned(),
        });
        let checked = index + 1;
        if checked == total || checked % 50 == 0 {
            progress(
                "qa",
                checked,
                total,
                run.failed_segment_ids.len(),
                warning_findings,
                blocking_findings,
                &format!("质量检查 {checked} / {total}"),
            );
        }
    }

    Ok((items, warning_findings, blocking_findings))
}

#[tauri::command]
fn export_translation_patch(input: ExportCommandInput) -> Result<ExportResult, String> {
    let parent = rfd::FileDialog::new()
        .set_title("选择汉化补丁导出位置")
        .pick_folder()
        .ok_or_else(|| "未选择导出位置".to_owned())?;
    let project_path = PathBuf::from(input.project_path);
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("game");
    let language_suffix = input
        .target_language
        .code
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .collect::<String>();
    let output = parent.join(format!("{project_name}-{language_suffix}"));
    export_path(
        &project_path,
        &input.items,
        &output,
        &input.target_language.code,
    )
}

fn export_path(
    project_path: &Path,
    items: &[TranslationItem],
    output_path: &Path,
    target_language: &str,
) -> Result<ExportResult, String> {
    let project = detect_game(project_path).map_err(|error| error.to_string())?;
    let plan = PatchPlan::capture(project).map_err(|error| error.to_string())?;
    let translations = items
        .iter()
        .map(|item| (item.id.clone(), item.target.clone()))
        .collect::<HashMap<_, _>>();
    let findings = items
        .iter()
        .flat_map(|item| {
            let mut findings = check_translation(&item.source, &item.target, None);
            if validate_control_codes(&item.source, &item.target).is_err() {
                findings.push(QaFinding {
                    code: QaCode::ControlCodeMismatch,
                    severity: QaSeverity::Blocking,
                });
            }
            findings
        })
        .collect::<Vec<_>>();
    let manifest = plan
        .export_for_language(&translations, &findings, output_path, target_language)
        .map_err(|error| error.to_string())?;
    Ok(ExportResult {
        output_path: output_path.to_string_lossy().into_owned(),
        file_count: manifest.files.len(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the desktop application event loop.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or run the platform event loop.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            select_and_scan_project,
            save_provider_configuration,
            translate_project,
            export_translation_patch
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GameTranslator");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn desktop_uses_the_core_product_name() {
        assert_eq!(game_translator_app_core::product_name(), "GameTranslator");
    }

    #[test]
    fn scans_a_real_rpg_maker_fixture() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");

        let result = super::scan_path(&fixture).unwrap();

        assert_eq!(result.engine, "RPG Maker MZ");
        assert_eq!(result.segment_count, 10);
        assert_eq!(result.project_path, fixture.to_string_lossy());
    }

    struct FakeProvider;

    impl game_translator_provider_core::TranslationProvider for FakeProvider {
        fn translate(
            &self,
            request: &game_translator_provider_core::TranslationRequest,
        ) -> Result<
            game_translator_provider_core::TranslationResponse,
            game_translator_provider_core::ProviderError,
        > {
            Ok(game_translator_provider_core::TranslationResponse {
                translations: request
                    .segments
                    .iter()
                    .map(|segment| game_translator_provider_core::TranslationOutput {
                        id: segment.id.clone(),
                        text: format!("译：{}", segment.text),
                    })
                    .collect(),
            })
        }
    }

    #[test]
    fn translates_a_real_fixture_and_restores_control_codes() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");

        let result = super::translate_path_with_provider(
            &fixture,
            &FakeProvider,
            "test-model",
            "Auto-detect the source language",
            "简体中文 (zh-CN)",
        )
        .unwrap();

        assert_eq!(result.items.len(), 10);
        let dialogue = result
            .items
            .iter()
            .find(|item| item.source.contains("\\V[1]"))
            .unwrap();
        assert_eq!(dialogue.target, "译：やっと着いた。 \\V[1]");
        assert_eq!(result.blocking_findings, 0);
    }

    #[test]
    fn exports_real_translations_to_a_separate_directory() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");
        let output = std::env::temp_dir().join(format!(
            "game-translator-command-export-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&output);
        let translated = super::translate_path_with_provider(
            &fixture,
            &FakeProvider,
            "test-model",
            "Auto-detect the source language",
            "简体中文 (zh-CN)",
        )
        .unwrap();

        let result = super::export_path(&fixture, &translated.items, &output, "zh-CN").unwrap();

        assert!(output.join("patch-manifest.json").is_file());
        assert!(output.join("data/Map001.json").is_file());
        assert!(result.file_count > 0);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    #[ignore = "requires an external Ren'Py distribution"]
    fn scans_an_external_renpy_distribution() {
        let root = std::env::var_os("GAME_TRANSLATOR_RENPY_FIXTURE")
            .map(PathBuf::from)
            .expect("set GAME_TRANSLATOR_RENPY_FIXTURE");

        let result = super::scan_path(&root).unwrap();

        assert_eq!(result.engine, "Ren'Py");
        assert!(result.segment_count > 100);
    }

    #[test]
    #[ignore = "requires an external Ren'Py distribution"]
    fn exports_an_external_renpy_distribution() {
        let root = std::env::var_os("GAME_TRANSLATOR_RENPY_FIXTURE")
            .map(PathBuf::from)
            .expect("set GAME_TRANSLATOR_RENPY_FIXTURE");
        let translated = super::translate_path_with_provider(
            &root,
            &FakeProvider,
            "test-model",
            "Auto-detect the source language",
            "English (en-US)",
        )
        .unwrap();
        let output = std::env::temp_dir().join(format!(
            "game-translator-renpy-export-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&output);

        let result = super::export_path(&root, &translated.items, &output, "en-US").unwrap();

        assert!(result.file_count > 0);
        assert!(output.join("game/tl/en-US/script.rpy").is_file());
        let rendered = std::fs::read_to_string(output.join("game/tl/en-US/script.rpy")).unwrap();
        assert!(rendered.contains("译："));
        let _ = std::fs::remove_dir_all(output);
    }
}
