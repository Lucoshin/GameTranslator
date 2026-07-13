// Tauri command adapters. Business policies belong in app-core.
mod dto;
mod events;

use dto::{
    DeletePatchHistoryInput, DesktopPreferences, ExportCommandInput, ExportResult,
    InstallCommandInput, InstallResult, LanguageInput, PatchHistoryEntry, ProgressMetrics,
    ProviderInput, ProviderMetadata, ResumableTask, ScanResult, TranslateCommandInput,
    TranslationExecution, TranslationItem, TranslationRunResult, UninstallCommandInput,
    UninstallResult,
};
use events::{emit_progress, emit_progress_with_metrics};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::Path,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use game_translator_app_core::{
    detect_content, detect_game, export_content, extract_content, AppError, AppErrorCode,
    CredentialStore, PatchFile, PatchManifest, PatchPlan, PerformanceSettings, ProviderFactory,
    ProviderSettings, WindowsCredentialStore, DEFAULT_PROVIDER_CREDENTIAL,
};
use game_translator_engine_core::Segment;
use game_translator_project_store::{CacheEntry, ProjectStore, TaskRecord, TaskState};
use game_translator_provider_core::TranslationProvider;
use game_translator_qa_core::{
    check_translation, protect_placeholders, restore_placeholders, validate_control_codes,
    ProtectedText, QaCode, QaFinding, QaSeverity,
};
use game_translator_translation_core::{
    RunControl, RunResult, TranslationOrchestrator, TranslationSegment,
};
use sha2::{Digest, Sha256};
use tauri::Manager;

#[tauri::command]
fn select_and_scan_project() -> Result<ScanResult, String> {
    let path = rfd::FileDialog::new()
        .set_title("选择游戏或模组目录")
        .pick_folder()
        .ok_or_else(|| "未选择内容目录".to_owned())?;
    scan_path(&path)
}

fn scan_path(path: &Path) -> Result<ScanResult, String> {
    let source = detect_content(path).map_err(|error| error.to_string())?;
    let segments = extract_content(&source).map_err(|error| error.to_string())?;
    let preview_items = segments
        .iter()
        .map(|segment| TranslationItem {
            id: segment.id.clone(),
            source: segment.source.clone(),
            target: String::new(),
            speaker: segment.context.speaker.clone(),
            source_file: segment.source_file.to_string_lossy().into_owned(),
            qa: "passed".to_owned(),
        })
        .collect();
    Ok(ScanResult {
        project_path: path.to_string_lossy().into_owned(),
        project_name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("未命名项目")
            .to_owned(),
        engine: content_label(source.format_id).to_owned(),
        segment_count: segments.len(),
        preview_items,
    })
}

fn content_label(format_id: &str) -> &'static str {
    match format_id {
        "game.rpgmaker.mv" => "RPG Maker MV",
        "game.rpgmaker.mz" => "RPG Maker MZ",
        "game.renpy" => "Ren'Py",
        "game.rimworld.mod" => "RimWorld 模组",
        _ => "内容来源",
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn save_provider_configuration(
    app: tauri::AppHandle,
    provider: ProviderInput,
) -> Result<(), String> {
    if provider.kind == "openai" {
        let mut credentials = WindowsCredentialStore::new("GameTranslator");
        if let Some(secret) = provider
            .api_key
            .as_deref()
            .filter(|secret| !secret.is_empty())
        {
            credentials
                .set(DEFAULT_PROVIDER_CREDENTIAL, secret)
                .map_err(|error| error.to_string())?;
        } else if credentials
            .get(DEFAULT_PROVIDER_CREDENTIAL)
            .map_err(|error| error.to_string())?
            .is_none()
        {
            return Err("OpenAI-compatible Provider 需要 API Key".to_owned());
        }
    }
    write_provider_metadata(&provider_metadata_path(&app)?, &provider)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn load_provider_configuration(app: tauri::AppHandle) -> Result<Option<ProviderMetadata>, String> {
    read_provider_metadata(&provider_metadata_path(&app)?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn load_desktop_preferences(app: tauri::AppHandle) -> Result<DesktopPreferences, String> {
    read_desktop_preferences(&desktop_preferences_path(&app)?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn save_desktop_preferences(
    app: tauri::AppHandle,
    preferences: DesktopPreferences,
) -> Result<(), String> {
    write_desktop_preferences(&desktop_preferences_path(&app)?, &preferences)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn list_resumable_tasks(app: tauri::AppHandle) -> Result<Vec<ResumableTask>, String> {
    let database_path = app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("translations.sqlite3");
    ProjectStore::open(&database_path)
        .and_then(|store| store.resumable_tasks())
        .map(|tasks| {
            tasks
                .into_iter()
                .map(|task| ResumableTask {
                    id: task.id,
                    project_path: task.project_path,
                    state: format!("{:?}", task.state).to_lowercase(),
                    total: task.total,
                    completed: task.completed,
                    failed: task.failed,
                    updated_at_unix_ms: task.updated_at_unix_ms,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn scan_project_path(project_path: String) -> Result<ScanResult, String> {
    scan_path(Path::new(&project_path))
}

#[tauri::command]
async fn translate_project(
    app: tauri::AppHandle,
    input: TranslateCommandInput,
) -> Result<TranslationRunResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || translate_command(&app, input))
        .await
        .map_err(|error| {
            AppError::new(
                AppErrorCode::Unexpected,
                format!("翻译任务异常终止: {error}"),
            )
        })?
}

fn translate_command(
    app: &tauri::AppHandle,
    input: TranslateCommandInput,
) -> Result<TranslationRunResult, AppError> {
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
                performance,
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
    let credentials = WindowsCredentialStore::new("GameTranslator");
    let provider = ProviderFactory::new(&credentials)
        .create(&ProviderSettings::new(&kind, &base_url, &model))?;
    let performance =
        PerformanceSettings::for_provider(&kind, performance.as_deref().unwrap_or("balanced"));
    let database_path = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::new(AppErrorCode::Io, error.to_string()))?
        .join("translations.sqlite3");
    let store = ProjectStore::open(&database_path).map_err(|error| error.to_string())?;
    let source_prompt = language_prompt(&source_language);
    let target_prompt = language_prompt(&target_language);
    let cache_scope = format!("v2\0{kind}\0{base_url}");
    translate_path_with_provider_and_progress(
        Path::new(&project_path),
        provider.as_ref(),
        &TranslationExecution {
            task_id: &run_id,
            model: &model,
            source_language: &source_prompt,
            target_language: &target_prompt,
            cache_scope: &cache_scope,
            store: Some(&store),
            batch_size: performance.batch_size,
            character_budget: performance.character_budget,
            initial_concurrency: performance.initial_concurrency,
            concurrency: performance.max_concurrency,
        },
        |phase, completed, total, failed, warnings, blocking, message, metrics| {
            emit_progress_with_metrics(
                app, &run_id, phase, completed, total, failed, warnings, blocking, message, metrics,
            );
        },
    )
    .map_err(AppError::from)
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
        &TranslationExecution {
            task_id: "test-run",
            model,
            source_language,
            target_language,
            cache_scope: "test",
            store: None,
            batch_size: 32,
            character_budget: 24_000,
            initial_concurrency: 1,
            concurrency: 1,
        },
        |_, _, _, _, _, _, _, _| {},
    )
}

#[allow(clippy::too_many_lines)]
fn translate_path_with_provider_and_progress<F>(
    path: &Path,
    provider: &dyn TranslationProvider,
    execution: &TranslationExecution<'_>,
    mut progress: F,
) -> Result<TranslationRunResult, String>
where
    F: FnMut(&str, usize, usize, usize, usize, usize, &str, ProgressMetrics),
{
    let source = detect_content(path).map_err(|error| error.to_string())?;
    let segments = extract_content(&source).map_err(|error| error.to_string())?;
    let total = segments.len();
    if let Some(store) = execution.store.filter(|_| !execution.task_id.is_empty()) {
        let task = TaskRecord::new(execution.task_id, path.to_string_lossy(), total);
        store
            .create_task(&task)
            .map_err(|error| error.to_string())?;
        store
            .update_task(execution.task_id, TaskState::Running, 0, 0, None)
            .map_err(|error| error.to_string())?;
    }
    progress(
        "translating",
        0,
        total,
        0,
        0,
        0,
        "文本提取完成，正在请求模型",
        ProgressMetrics::default(),
    );
    let mut protected = HashMap::new();
    let (fingerprints, cached) = load_cached_translations(&segments, execution);
    if !cached.is_empty() {
        progress(
            "translating",
            cached.len(),
            total,
            0,
            0,
            0,
            &format!("命中 {} 条翻译缓存", cached.len()),
            ProgressMetrics::default(),
        );
    }
    let translation_segments = segments
        .iter()
        .map(|segment| {
            let protected_text = protect_placeholders(&segment.source);
            let source = protected_text.text.clone();
            protected.insert(segment.id.clone(), protected_text);
            TranslationSegment::new(&segment.id, segment.source_file.to_string_lossy(), source)
        })
        .collect::<Vec<_>>();
    let orchestrator = TranslationOrchestrator::new(
        provider,
        execution.model,
        execution.source_language,
        execution.target_language,
        execution.batch_size,
    )
    .with_batch_character_budget(execution.character_budget)
    .with_adaptive_concurrency(execution.initial_concurrency, execution.concurrency);
    let translation_started = Instant::now();
    let cached_count = cached.len();
    let persistence_error = RefCell::new(None::<String>);
    let run = orchestrator.run_with_progress(
        &translation_segments,
        &cached,
        RunControl::Running,
        |current| {
            let completed = current.translations.len() + current.failed_segment_ids.len();
            if let Some(store) = execution.store.filter(|_| !execution.task_id.is_empty()) {
                let snapshot = serde_json::json!({
                    "phase": "translating",
                    "completedBatches": current.completed_batches,
                    "totalBatches": current.total_batches,
                    "activeRequests": current.active_requests
                })
                .to_string();
                if let Err(error) = store.update_task_progress(
                    execution.task_id,
                    completed,
                    current.failed_segment_ids.len(),
                    Some(&snapshot),
                ) {
                    *persistence_error.borrow_mut() = Some(error.to_string());
                }
            }
            let message = translation_progress_message(current);
            let metrics = calculate_progress_metrics(
                current,
                cached_count,
                completed,
                total,
                translation_started.elapsed(),
            );
            progress(
                "translating",
                completed,
                total,
                current.failed_segment_ids.len(),
                0,
                0,
                &message,
                metrics,
            );
        },
    );
    if let Some(error) = persistence_error.into_inner() {
        return Err(error);
    }
    progress(
        "qa",
        0,
        total,
        run.failed_segment_ids.len(),
        0,
        0,
        "模型翻译结束，正在执行质量检查",
        ProgressMetrics::default(),
    );
    let (items, warning_findings, blocking_findings, placeholder_failed_ids) = restore_and_check(
        segments,
        &run,
        &protected,
        &cached,
        &fingerprints,
        execution.store,
        &mut progress,
    )?;

    let mut failed_segment_ids = run.failed_segment_ids;
    failed_segment_ids.extend(placeholder_failed_ids);
    progress(
        "completed",
        total,
        total,
        failed_segment_ids.len(),
        warning_findings,
        blocking_findings,
        "任务完成，可以校对或导出",
        ProgressMetrics::default(),
    );

    let result = TranslationRunResult {
        items,
        warning_findings,
        blocking_findings,
        failed_segment_ids,
    };
    if let Some(store) = execution.store.filter(|_| !execution.task_id.is_empty()) {
        let snapshot = serde_json::to_string(&result).map_err(|error| error.to_string())?;
        store
            .update_task(
                execution.task_id,
                TaskState::Completed,
                total,
                result.failed_segment_ids.len(),
                Some(&snapshot),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(result)
}

fn translation_progress_message(current: &RunResult) -> String {
    if current.completed_batches == 0 {
        return format!(
            "已启动 {} 路并发请求 · 共 {} 个批次",
            current.active_requests, current.total_batches
        );
    }
    let seconds = current.last_batch_millis / 1_000;
    let tenths = current.last_batch_millis % 1_000 / 100;
    format!(
        "批次 {}/{} · 活动请求 {} · 最近耗时 {seconds}.{tenths} 秒",
        current.completed_batches, current.total_batches, current.active_requests
    )
}

fn calculate_progress_metrics(
    current: &RunResult,
    cached_count: usize,
    completed: usize,
    total: usize,
    elapsed: std::time::Duration,
) -> ProgressMetrics {
    let network_completed =
        current.translations.len().saturating_sub(cached_count) + current.failed_segment_ids.len();
    let count = u32::try_from(network_completed).unwrap_or(u32::MAX);
    let throughput = f64::from(count) / elapsed.as_secs_f64().max(0.001);
    let remaining = u128::try_from(total.saturating_sub(completed)).unwrap_or(u128::MAX);
    let eta_millis = if network_completed == 0 {
        0
    } else {
        remaining
            .saturating_mul(elapsed.as_millis())
            .checked_div(u128::try_from(network_completed).unwrap_or(u128::MAX))
            .unwrap_or(0)
    };
    ProgressMetrics {
        concurrency: current.concurrency_limit,
        throughput,
        eta_seconds: usize::try_from(eta_millis.div_ceil(1_000)).unwrap_or(usize::MAX),
    }
}

fn restore_and_check<F>(
    segments: Vec<Segment>,
    run: &RunResult,
    protected: &HashMap<String, ProtectedText>,
    cached: &HashMap<String, String>,
    fingerprints: &HashMap<String, String>,
    store: Option<&ProjectStore>,
    progress: &mut F,
) -> Result<(Vec<TranslationItem>, usize, usize, Vec<String>), String>
where
    F: FnMut(&str, usize, usize, usize, usize, usize, &str, ProgressMetrics),
{
    let total = segments.len();
    let mut items = Vec::with_capacity(run.translations.len());
    let mut warning_findings = 0;
    let mut blocking_findings = 0;
    let mut placeholder_failed_ids = Vec::new();
    for (index, segment) in segments.into_iter().enumerate() {
        let Some(translated) = run.translations.get(&segment.id) else {
            continue;
        };
        let restored = restore_placeholders(
            protected
                .get(&segment.id)
                .ok_or_else(|| format!("缺少占位符映射: {}", segment.id))?,
            translated,
        );
        let Ok(target) = restored else {
            blocking_findings += 1;
            placeholder_failed_ids.push(segment.id.clone());
            items.push(TranslationItem {
                id: segment.id,
                source: segment.source.clone(),
                target: segment.source,
                speaker: segment.context.speaker,
                source_file: segment.source_file.to_string_lossy().into_owned(),
                qa: "blocking".to_owned(),
            });
            continue;
        };
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
        if item_qa != "blocking" && !cached.contains_key(&segment.id) {
            if let (Some(store), Some(fingerprint)) = (store, fingerprints.get(&segment.id)) {
                store
                    .put_cache(&CacheEntry {
                        input_fingerprint: fingerprint.clone(),
                        translation: translated.clone(),
                    })
                    .map_err(|error| error.to_string())?;
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
                ProgressMetrics::default(),
            );
        }
    }

    Ok((
        items,
        warning_findings,
        blocking_findings,
        placeholder_failed_ids,
    ))
}

fn translation_fingerprint(segment: &Segment, execution: &TranslationExecution<'_>) -> String {
    let mut digest = Sha256::new();
    for value in [
        "game-translator-prompt-v2",
        execution.cache_scope,
        execution.model,
        execution.source_language,
        execution.target_language,
        &segment.id,
        &segment.source,
        segment.context.speaker.as_deref().unwrap_or(""),
        segment.context.previous_text.as_deref().unwrap_or(""),
        segment.context.next_text.as_deref().unwrap_or(""),
    ] {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

fn load_cached_translations(
    segments: &[Segment],
    execution: &TranslationExecution<'_>,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let fingerprints = segments
        .iter()
        .map(|segment| {
            (
                segment.id.clone(),
                translation_fingerprint(segment, execution),
            )
        })
        .collect::<HashMap<_, _>>();
    let cached = execution.store.map_or_else(HashMap::new, |store| {
        fingerprints
            .iter()
            .filter_map(|(id, fingerprint)| {
                store
                    .cached_translation(fingerprint)
                    .ok()
                    .flatten()
                    .map(|translation| (id.clone(), translation))
            })
            .collect()
    });
    (fingerprints, cached)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn export_translation_patch(
    app: tauri::AppHandle,
    input: ExportCommandInput,
) -> Result<ExportResult, String> {
    let parent = rfd::FileDialog::new()
        .set_title("选择翻译补丁导出位置")
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
    let result = export_path(
        &project_path,
        &input.items,
        &output,
        &input.target_language.code,
    )?;
    record_patch_export(
        &patch_history_path(&app)?,
        &project_path,
        &output,
        &input.target_language.code,
        result.file_count,
        unix_time_millis()?,
    )?;
    Ok(result)
}

fn export_path(
    project_path: &Path,
    items: &[TranslationItem],
    output_path: &Path,
    target_language: &str,
) -> Result<ExportResult, String> {
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
    if findings
        .iter()
        .any(|finding| finding.severity == QaSeverity::Blocking)
    {
        return Err("存在阻断性质量问题，无法导出".to_owned());
    }
    let source = detect_content(project_path).map_err(|error| error.to_string())?;
    if source.format_id == "game.rimworld.mod" {
        let exported = export_content(&source, &translations, output_path, target_language)
            .map_err(|error| error.to_string())?;
        let manifest = write_content_patch_manifest(project_path, output_path, &exported.files)?;
        return Ok(ExportResult {
            output_path: output_path.to_string_lossy().into_owned(),
            file_count: manifest.files.len(),
        });
    }
    let project = detect_game(project_path).map_err(|error| error.to_string())?;
    let plan = PatchPlan::capture(project).map_err(|error| error.to_string())?;
    let manifest = plan
        .export_for_language(&translations, &findings, output_path, target_language)
        .map_err(|error| error.to_string())?;
    Ok(ExportResult {
        output_path: output_path.to_string_lossy().into_owned(),
        file_count: manifest.files.len(),
    })
}

fn write_content_patch_manifest(
    project_path: &Path,
    output_path: &Path,
    files: &[PathBuf],
) -> Result<PatchManifest, String> {
    let mut manifest_files = Vec::with_capacity(files.len());
    for target_path in files {
        let relative_path = target_path
            .strip_prefix(output_path)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        let source_path = project_path.join(&relative_path);
        manifest_files.push(PatchFile {
            relative_path,
            source_sha256: if source_path.is_file() {
                file_sha256(&source_path)?
            } else {
                String::new()
            },
            target_sha256: file_sha256(target_path)?,
        });
    }
    let manifest = PatchManifest {
        format_version: 1,
        files: manifest_files,
    };
    let manifest_path = output_path.join("patch-manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(manifest)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn install_translation_patch(
    app: tauri::AppHandle,
    input: InstallCommandInput,
) -> Result<InstallResult, String> {
    let InstallCommandInput {
        project_path,
        patch_path,
        target_language,
    } = input;
    let project_path = PathBuf::from(project_path);
    let patch_path = PathBuf::from(patch_path);
    let result = install_patch_path(&project_path, &patch_path, &target_language.code)?;
    record_patch_installation(
        &patch_history_path(&app)?,
        &project_path,
        &patch_path,
        &target_language.code,
        result.file_count,
        unix_time_millis()?,
    )?;
    Ok(result)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri deserializes command payloads by value.
fn list_patch_history(
    app: tauri::AppHandle,
    project_path: Option<String>,
) -> Result<Vec<PatchHistoryEntry>, String> {
    let history_path = patch_history_path(&app)?;
    match project_path {
        Some(project_path) => patch_history_for_project(&history_path, Path::new(&project_path)),
        None => all_patch_history(&history_path),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn uninstall_translation_patch(
    app: tauri::AppHandle,
    input: UninstallCommandInput,
) -> Result<UninstallResult, String> {
    let history_path = patch_history_path(&app)?;
    let project_path = PathBuf::from(input.project_path);
    let entry = patch_history_for_project(&history_path, &project_path)?
        .into_iter()
        .find(|entry| entry.id == input.id)
        .ok_or_else(|| "未找到该项目的补丁历史记录".to_owned())?;
    if entry.installed_at_unix_ms.is_none() {
        return Err("该补丁没有安装记录，无法卸载".to_owned());
    }
    let result = uninstall_patch_path(&project_path, Path::new(&entry.patch_path))?;
    clear_patch_installation(&history_path, &project_path, Path::new(&entry.patch_path))?;
    Ok(result)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the application handle by value.
fn delete_patch_history_entry(
    app: tauri::AppHandle,
    input: DeletePatchHistoryInput,
) -> Result<(), String> {
    let history_path = patch_history_path(&app)?;
    remove_patch_history_entry(&history_path, Path::new(&input.project_path), &input.id)
}

fn install_patch_path(
    project_path: &Path,
    patch_path: &Path,
    target_language: &str,
) -> Result<InstallResult, String> {
    let source = detect_content(project_path).map_err(|error| error.to_string())?;
    if source.format_id != "game.renpy" && source.format_id != "game.rimworld.mod" {
        return Err("当前内容类型仅支持导出独立补丁目录".into());
    }
    let manifest = verified_patch_manifest(patch_path)?;
    if source.format_id == "game.rimworld.mod" {
        validate_rimworld_language_manifest(&manifest)?;
    }
    let backup_root = patch_path.join("backup");
    for file in &manifest.files {
        let source = patch_path.join(&file.relative_path);
        let destination = project_path.join(&file.relative_path);
        if destination.exists() {
            let backup = backup_root.join(&file.relative_path);
            if let Some(parent) = backup.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::copy(&destination, backup).map_err(|error| error.to_string())?;
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::copy(source, destination).map_err(|error| error.to_string())?;
    }
    let installed_path = if source.format_id == "game.rimworld.mod" {
        project_path.join("Languages/ChineseSimplified")
    } else {
        let language = game_translator_engine_renpy::language_identifier(target_language);
        project_path.join("game/tl").join(language)
    };
    Ok(InstallResult {
        installed_path: installed_path.to_string_lossy().into_owned(),
        file_count: manifest.files.len(),
    })
}

fn uninstall_patch_path(project_path: &Path, patch_path: &Path) -> Result<UninstallResult, String> {
    let source = detect_content(project_path).map_err(|error| error.to_string())?;
    if source.format_id != "game.renpy" && source.format_id != "game.rimworld.mod" {
        return Err("当前内容类型不支持自动卸载".into());
    }
    let manifest = verified_patch_manifest(patch_path)?;
    if source.format_id == "game.rimworld.mod" {
        validate_rimworld_language_manifest(&manifest)?;
    }
    let backup_root = patch_path.join("backup");
    let mut actions = Vec::new();

    for file in &manifest.files {
        let destination = project_path.join(&file.relative_path);
        let backup = backup_root.join(&file.relative_path);
        if backup.is_file() {
            actions.push((destination, Some(backup)));
        } else if destination.exists() {
            let actual_hash = file_sha256(&destination)?;
            if actual_hash != file.target_sha256 {
                return Err(format!(
                    "拒绝卸载：安装文件已被修改: {}",
                    file.relative_path.display()
                ));
            }
            actions.push((destination, None));
        }
    }

    let mut restored_file_count = 0;
    let mut removed_file_count = 0;
    for (destination, backup) in actions {
        if let Some(backup) = backup {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::copy(backup, destination).map_err(|error| error.to_string())?;
            restored_file_count += 1;
        } else {
            std::fs::remove_file(destination).map_err(|error| error.to_string())?;
            removed_file_count += 1;
        }
    }
    Ok(UninstallResult {
        restored_file_count,
        removed_file_count,
    })
}

fn validate_rimworld_language_manifest(manifest: &PatchManifest) -> Result<(), String> {
    for file in &manifest.files {
        file.relative_path
            .strip_prefix("Languages/ChineseSimplified")
            .map_err(|_| {
                format!(
                    "RimWorld 语言包包含无效路径: {}",
                    file.relative_path.display()
                )
            })?;
    }
    Ok(())
}

fn verified_patch_manifest(patch_path: &Path) -> Result<PatchManifest, String> {
    let manifest_path = patch_path.join("patch-manifest.json");
    let manifest: PatchManifest = serde_json::from_slice(
        &std::fs::read(&manifest_path).map_err(|error| format!("读取补丁清单失败: {error}"))?,
    )
    .map_err(|error| format!("补丁清单无效: {error}"))?;
    if manifest.format_version != 1 {
        return Err(format!("不支持的补丁格式版本: {}", manifest.format_version));
    }
    for file in &manifest.files {
        validate_patch_relative_path(&file.relative_path)?;
        let source = patch_path.join(&file.relative_path);
        let actual_hash = file_sha256(&source)?;
        if actual_hash != file.target_sha256 {
            return Err(format!(
                "补丁文件校验失败: {}",
                file.relative_path.display()
            ));
        }
    }
    Ok(manifest)
}

fn record_patch_export(
    history_path: &Path,
    project_path: &Path,
    patch_path: &Path,
    target_language: &str,
    file_count: usize,
    exported_at_unix_ms: u64,
) -> Result<(), String> {
    let mut entries = read_patch_history(history_path)?;
    let id = patch_history_id(project_path, patch_path);
    if let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) {
        target_language.clone_into(&mut entry.target_language);
        entry.file_count = file_count;
        entry.exported_at_unix_ms = exported_at_unix_ms;
    } else {
        entries.push(PatchHistoryEntry {
            id,
            project_path: project_path.to_string_lossy().into_owned(),
            patch_path: patch_path.to_string_lossy().into_owned(),
            target_language: target_language.to_owned(),
            file_count,
            exported_at_unix_ms,
            installed_at_unix_ms: None,
        });
    }
    write_patch_history(history_path, &entries)
}

fn record_patch_installation(
    history_path: &Path,
    project_path: &Path,
    patch_path: &Path,
    target_language: &str,
    file_count: usize,
    installed_at_unix_ms: u64,
) -> Result<(), String> {
    let mut entries = read_patch_history(history_path)?;
    let id = patch_history_id(project_path, patch_path);
    if let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) {
        target_language.clone_into(&mut entry.target_language);
        entry.file_count = file_count;
        entry.installed_at_unix_ms = Some(installed_at_unix_ms);
    } else {
        entries.push(PatchHistoryEntry {
            id,
            project_path: project_path.to_string_lossy().into_owned(),
            patch_path: patch_path.to_string_lossy().into_owned(),
            target_language: target_language.to_owned(),
            file_count,
            exported_at_unix_ms: installed_at_unix_ms,
            installed_at_unix_ms: Some(installed_at_unix_ms),
        });
    }
    write_patch_history(history_path, &entries)
}

fn clear_patch_installation(
    history_path: &Path,
    project_path: &Path,
    patch_path: &Path,
) -> Result<(), String> {
    let mut entries = read_patch_history(history_path)?;
    let id = patch_history_id(project_path, patch_path);
    let entry = entries
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| "未找到补丁历史记录".to_owned())?;
    entry.installed_at_unix_ms = None;
    write_patch_history(history_path, &entries)
}

fn remove_patch_history_entry(
    history_path: &Path,
    project_path: &Path,
    id: &str,
) -> Result<(), String> {
    let mut entries = read_patch_history(history_path)?;
    let entry = entries
        .iter()
        .find(|entry| entry.project_path == project_path.to_string_lossy() && entry.id == id)
        .ok_or_else(|| "未找到该项目的补丁历史记录".to_owned())?;
    if entry.installed_at_unix_ms.is_some() {
        return Err("请先卸载补丁，再删除历史记录".to_owned());
    }
    entries.retain(|entry| entry.id != id);
    write_patch_history(history_path, &entries)
}

fn patch_history_for_project(
    history_path: &Path,
    project_path: &Path,
) -> Result<Vec<PatchHistoryEntry>, String> {
    let project_path = project_path.to_string_lossy();
    let mut entries = read_patch_history(history_path)?
        .into_iter()
        .filter(|entry| entry.project_path == project_path)
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .installed_at_unix_ms
            .unwrap_or(right.exported_at_unix_ms)
            .cmp(
                &left
                    .installed_at_unix_ms
                    .unwrap_or(left.exported_at_unix_ms),
            )
    });
    Ok(entries)
}

fn all_patch_history(history_path: &Path) -> Result<Vec<PatchHistoryEntry>, String> {
    let mut entries = read_patch_history(history_path)?;
    entries.sort_by(|left, right| {
        right
            .installed_at_unix_ms
            .unwrap_or(right.exported_at_unix_ms)
            .cmp(
                &left
                    .installed_at_unix_ms
                    .unwrap_or(left.exported_at_unix_ms),
            )
    });
    Ok(entries)
}

fn read_patch_history(history_path: &Path) -> Result<Vec<PatchHistoryEntry>, String> {
    match std::fs::read(history_path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|error| format!("补丁历史记录无效: {error}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("读取补丁历史失败: {error}")),
    }
}

fn write_patch_history(history_path: &Path, entries: &[PatchHistoryEntry]) -> Result<(), String> {
    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("创建历史目录失败: {error}"))?;
    }
    let content = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("序列化补丁历史失败: {error}"))?;
    std::fs::write(history_path, content).map_err(|error| format!("写入补丁历史失败: {error}"))
}

fn patch_history_id(project_path: &Path, patch_path: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(project_path.to_string_lossy().as_bytes());
    digest.update([0]);
    digest.update(patch_path.to_string_lossy().as_bytes());
    format!("{:x}", digest.finalize())
}

fn provider_metadata_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|directory| directory.join("provider-configuration.json"))
        .map_err(|error| format!("读取应用数据目录失败: {error}"))
}

fn write_provider_metadata(path: &Path, provider: &ProviderInput) -> Result<(), String> {
    let metadata = ProviderMetadata {
        kind: provider.kind.clone(),
        base_url: provider.base_url.clone(),
        model: provider.model.clone(),
        performance: provider.performance.clone(),
    };
    let parent = path
        .parent()
        .ok_or_else(|| "模型配置路径缺少父目录".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建模型配置目录失败: {error}"))?;
    let rendered = serde_json::to_vec_pretty(&metadata)
        .map_err(|error| format!("序列化模型配置失败: {error}"))?;
    std::fs::write(path, rendered).map_err(|error| format!("写入模型配置失败: {error}"))
}

fn read_provider_metadata(path: &Path) -> Result<Option<ProviderMetadata>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    serde_json::from_slice(
        &std::fs::read(path).map_err(|error| format!("读取模型配置失败: {error}"))?,
    )
    .map(Some)
    .map_err(|error| format!("模型配置无效: {error}"))
}

fn patch_history_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|directory| directory.join("patch-history.json"))
        .map_err(|error| format!("读取应用数据目录失败: {error}"))
}

fn desktop_preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|directory| directory.join("desktop-preferences.json"))
        .map_err(|error| format!("读取应用数据目录失败: {error}"))
}

fn write_desktop_preferences(path: &Path, preferences: &DesktopPreferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "桌面偏好路径缺少父目录".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建偏好目录失败: {error}"))?;
    let rendered = serde_json::to_vec_pretty(preferences)
        .map_err(|error| format!("序列化桌面偏好失败: {error}"))?;
    std::fs::write(path, rendered).map_err(|error| format!("写入桌面偏好失败: {error}"))
}

fn read_desktop_preferences(path: &Path) -> Result<DesktopPreferences, String> {
    if !path.is_file() {
        return Ok(DesktopPreferences::default());
    }
    serde_json::from_slice(
        &std::fs::read(path).map_err(|error| format!("读取桌面偏好失败: {error}"))?,
    )
    .map_err(|error| format!("桌面偏好无效: {error}"))
}

fn unix_time_millis() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("读取系统时间失败: {error}"))?
        .as_millis()
        .try_into()
        .map_err(|_| "系统时间超出支持范围".to_owned())
}

fn validate_patch_relative_path(path: &Path) -> Result<(), String> {
    if path.is_absolute()
        || path.components().any(|component| {
            !matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        return Err(format!("补丁包含不安全路径: {}", path.display()));
    }
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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
            load_provider_configuration,
            load_desktop_preferences,
            save_desktop_preferences,
            list_resumable_tasks,
            scan_project_path,
            translate_project,
            export_translation_patch,
            install_translation_patch,
            list_patch_history,
            uninstall_translation_patch,
            delete_patch_history_entry
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GameTranslator");
}

#[cfg(test)]
mod tests {
    use game_translator_app_core::extract_game;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{fs, path::Path, path::PathBuf};

    fn copy_directory(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = destination.join(entry.file_name());
            if entry.path().is_dir() {
                copy_directory(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    #[test]
    fn desktop_uses_the_core_product_name() {
        assert_eq!(game_translator_app_core::product_name(), "GameTranslator");
    }

    #[test]
    fn desktop_capability_allows_progress_event_listening() {
        let capability = include_str!("../../capabilities/default.json");
        assert!(capability.contains("core:event:allow-listen"));
        assert!(capability.contains("core:event:allow-unlisten"));
    }

    #[test]
    fn deepseek_fast_mode_uses_small_batches_and_adaptive_headroom() {
        assert_eq!(
            game_translator_app_core::PerformanceSettings::for_provider("openai", "fast"),
            game_translator_app_core::PerformanceSettings::new(16, 8_000, 16, 128)
        );
    }

    #[test]
    #[ignore = "uses the saved DeepSeek credential and incurs API usage"]
    fn benchmarks_saved_deepseek_configuration() {
        use game_translator_app_core::{CredentialStore, WindowsCredentialStore};
        use game_translator_provider_core::{
            OpenAiCompatibleProvider, TranslationInput, TranslationProvider, TranslationRequest,
        };
        use std::time::Instant;

        let root = std::env::var_os("GAME_TRANSLATOR_RENPY_FIXTURE")
            .map(PathBuf::from)
            .expect("set GAME_TRANSLATOR_RENPY_FIXTURE");
        let project = super::detect_game(&root).unwrap();
        let segments = extract_game(&project).unwrap();
        let api_key = WindowsCredentialStore::new("GameTranslator")
            .get("default-provider")
            .unwrap()
            .expect("save a DeepSeek API key first");
        let provider = OpenAiCompatibleProvider::new("https://api.deepseek.com", api_key)
            .with_user_id("game-translator-benchmark");
        let request = TranslationRequest {
            model: "deepseek-v4-flash".into(),
            source_language: "Auto-detect the source language".into(),
            target_language: "简体中文 (zh-CN)".into(),
            segments: segments
                .iter()
                .take(16)
                .enumerate()
                .map(|(index, segment)| TranslationInput {
                    id: index.to_string(),
                    text: super::protect_placeholders(&segment.source).text,
                })
                .collect(),
        };

        let started = Instant::now();
        let response = provider.translate(&request).unwrap();
        let elapsed = started.elapsed();

        eprintln!(
            "DEEPSEEK_BENCHMARK segments={} elapsed_ms={} segments_per_second={:.2}",
            response.translations.len(),
            elapsed.as_millis(),
            f64::from(u32::try_from(response.translations.len()).unwrap()) / elapsed.as_secs_f64()
        );
        assert_eq!(response.translations.len(), request.segments.len());

        let concurrent_request = TranslationRequest {
            segments: request.segments[..8].to_vec(),
            ..request
        };
        let concurrent_started = Instant::now();
        let translated = std::thread::scope(|scope| {
            let handles = (0..8)
                .map(|_| {
                    let request = concurrent_request.clone();
                    let provider = &provider;
                    scope.spawn(move || provider.translate(&request).unwrap().translations.len())
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .sum::<usize>()
        });
        let concurrent_elapsed = concurrent_started.elapsed();
        eprintln!(
            "DEEPSEEK_CONCURRENT_BENCHMARK requests=8 segments={} elapsed_ms={} segments_per_second={:.2}",
            translated,
            concurrent_elapsed.as_millis(),
            f64::from(u32::try_from(translated).unwrap()) / concurrent_elapsed.as_secs_f64()
        );
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

    #[test]
    fn scan_returns_source_text_for_review_before_translation() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");

        let result = super::scan_path(&fixture).unwrap();

        assert!(!result.preview_items.is_empty());
        assert!(result
            .preview_items
            .iter()
            .all(|item| item.target.is_empty()));
    }

    #[test]
    fn rimworld_mod_scans_exports_installs_and_uninstalls_a_language_pack() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rimworld-mod-minimal");
        let root = std::env::temp_dir().join(format!(
            "game-translator-rimworld-project-{}",
            std::process::id()
        ));
        let output = std::env::temp_dir().join(format!(
            "game-translator-rimworld-patch-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
        copy_directory(&fixture, &root);

        let scanned = super::scan_path(&root).unwrap();
        assert_eq!(scanned.engine, "RimWorld 模组");
        assert_eq!(scanned.segment_count, 3);
        let items = scanned
            .preview_items
            .into_iter()
            .map(|mut item| {
                item.target = format!("译文：{}", item.source);
                item
            })
            .collect::<Vec<_>>();

        let exported = super::export_path(&root, &items, &output, "zh-CN").unwrap();
        let relative = PathBuf::from("Languages/ChineseSimplified/Keyed/Example.xml");
        assert!(output.join("patch-manifest.json").is_file());
        assert!(output.join(&relative).is_file());
        assert!(exported.file_count >= 2);

        let installed = super::install_patch_path(&root, &output, "zh-CN").unwrap();
        assert_eq!(installed.file_count, exported.file_count);
        assert!(root.join(&relative).is_file());

        let uninstalled = super::uninstall_patch_path(&root, &output).unwrap();
        assert_eq!(uninstalled.restored_file_count, 0);
        assert!(uninstalled.removed_file_count >= 2);
        assert!(!root.join(&relative).exists());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(output);
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

    struct CountingProvider(AtomicUsize);

    impl game_translator_provider_core::TranslationProvider for CountingProvider {
        fn translate(
            &self,
            request: &game_translator_provider_core::TranslationRequest,
        ) -> Result<
            game_translator_provider_core::TranslationResponse,
            game_translator_provider_core::ProviderError,
        > {
            self.0.fetch_add(1, Ordering::SeqCst);
            FakeProvider.translate(request)
        }
    }

    struct OneMalformedPlaceholderProvider;

    impl game_translator_provider_core::TranslationProvider for OneMalformedPlaceholderProvider {
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
                        text: if segment.text.contains("<ph") {
                            "<ph broken/>".to_owned()
                        } else {
                            format!("译：{}", segment.text)
                        },
                    })
                    .collect(),
            })
        }
    }

    #[test]
    fn isolates_malformed_placeholder_output_instead_of_aborting_the_run() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");

        let result = super::translate_path_with_provider(
            &fixture,
            &OneMalformedPlaceholderProvider,
            "test-model",
            "auto",
            "ja-JP",
        )
        .unwrap();

        assert_eq!(result.items.len(), 10);
        assert!(result.blocking_findings > 0);
        assert!(!result.failed_segment_ids.is_empty());
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
    fn reuses_persistent_translations_without_calling_the_provider_again() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/rpgmaker-mz-dialogue");
        let database = std::env::temp_dir().join(format!(
            "game-translator-desktop-cache-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&database);
        let provider = CountingProvider(AtomicUsize::new(0));
        {
            let store = game_translator_project_store::ProjectStore::open(&database).unwrap();
            let execution = super::TranslationExecution {
                task_id: "",
                model: "test-model",
                source_language: "auto",
                target_language: "zh-CN",
                cache_scope: "test-cache-v1",
                store: Some(&store),
                batch_size: 4,
                character_budget: 10_000,
                initial_concurrency: 2,
                concurrency: 2,
            };
            super::translate_path_with_provider_and_progress(
                &fixture,
                &provider,
                &execution,
                |_, _, _, _, _, _, _, _| {},
            )
            .unwrap();
            let first_run_calls = provider.0.load(Ordering::SeqCst);

            super::translate_path_with_provider_and_progress(
                &fixture,
                &provider,
                &execution,
                |_, _, _, _, _, _, _, _| {},
            )
            .unwrap();

            assert!(first_run_calls > 0);
            assert_eq!(provider.0.load(Ordering::SeqCst), first_run_calls);
        }
        let _ = std::fs::remove_file(database);
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
    fn installs_a_verified_renpy_patch_into_the_game() {
        use sha2::{Digest, Sha256};

        let root = std::env::temp_dir().join(format!(
            "game-translator-install-project-{}",
            std::process::id()
        ));
        let patch = std::env::temp_dir().join(format!(
            "game-translator-install-patch-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&patch);
        std::fs::create_dir_all(root.join("renpy")).unwrap();
        std::fs::create_dir_all(root.join("game")).unwrap();
        std::fs::write(root.join("Mayfly.py"), "").unwrap();
        let relative = PathBuf::from("game/tl/ja_JP/script.rpy");
        std::fs::create_dir_all(patch.join(relative.parent().unwrap())).unwrap();
        let content = "translate ja_JP start:\n    \"こんにちは\"\n";
        std::fs::write(patch.join(&relative), content).unwrap();
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        std::fs::write(
            patch.join("patch-manifest.json"),
            format!(
                r#"{{"format_version":1,"files":[{{"relative_path":"game/tl/ja_JP/script.rpy","source_sha256":"","target_sha256":"{hash}"}}]}}"#
            ),
        )
        .unwrap();

        let result = super::install_patch_path(&root, &patch, "ja-JP").unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(
            std::fs::read_to_string(root.join(&relative)).unwrap(),
            content
        );
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(patch);
    }

    #[test]
    fn patch_history_tracks_exports_and_installations_per_project() {
        let history = std::env::temp_dir().join(format!(
            "game-translator-patch-history-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&history);
        let first_project = PathBuf::from("D:/Games/First");
        let second_project = PathBuf::from("D:/Games/Second");
        let first_patch = PathBuf::from("D:/Patches/First-zh-CN");
        let second_patch = PathBuf::from("D:/Patches/Second-zh-CN");

        super::record_patch_export(&history, &first_project, &first_patch, "zh-CN", 2, 10).unwrap();
        super::record_patch_export(&history, &second_project, &second_patch, "zh-CN", 1, 20)
            .unwrap();
        super::record_patch_installation(&history, &first_project, &first_patch, "zh-CN", 2, 30)
            .unwrap();

        let entries = super::patch_history_for_project(&history, &first_project).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].patch_path, first_patch.to_string_lossy());
        assert_eq!(entries[0].file_count, 2);
        assert_eq!(entries[0].installed_at_unix_ms, Some(30));
        let _ = std::fs::remove_file(history);
    }

    #[test]
    fn clearing_an_installation_keeps_its_export_in_history() {
        let history = std::env::temp_dir().join(format!(
            "game-translator-patch-history-clear-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&history);
        let project = PathBuf::from("D:/Games/First");
        let patch = PathBuf::from("D:/Patches/First-zh-CN");
        super::record_patch_export(&history, &project, &patch, "zh-CN", 2, 10).unwrap();
        super::record_patch_installation(&history, &project, &patch, "zh-CN", 2, 20).unwrap();

        super::clear_patch_installation(&history, &project, &patch).unwrap();

        let entries = super::patch_history_for_project(&history, &project).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].installed_at_unix_ms, None);
        let _ = std::fs::remove_file(history);
    }

    #[test]
    fn deletes_only_an_uninstalled_patch_history_entry() {
        let history = std::env::temp_dir().join(format!(
            "game-translator-patch-history-delete-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&history);
        let project = PathBuf::from("D:/Games/First");
        let patch = PathBuf::from("D:/Patches/First-zh-CN");
        super::record_patch_export(&history, &project, &patch, "zh-CN", 2, 10).unwrap();

        super::remove_patch_history_entry(
            &history,
            &project,
            &super::patch_history_id(&project, &patch),
        )
        .unwrap();

        assert!(super::patch_history_for_project(&history, &project)
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_file(history);
    }

    #[test]
    fn persists_provider_metadata_without_the_api_key() {
        let path = std::env::temp_dir().join(format!(
            "game-translator-provider-configuration-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let provider = super::ProviderInput {
            kind: "openai".to_owned(),
            base_url: "https://api.deepseek.com".to_owned(),
            model: "deepseek-chat".to_owned(),
            api_key: Some("secret-value".to_owned()),
            performance: Some("fast".to_owned()),
        };

        super::write_provider_metadata(&path, &provider).unwrap();

        let saved = super::read_provider_metadata(&path).unwrap().unwrap();
        assert_eq!(saved.kind, "openai");
        assert_eq!(saved.base_url, "https://api.deepseek.com");
        assert_eq!(saved.model, "deepseek-chat");
        assert_eq!(saved.performance.as_deref(), Some("fast"));
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains("secret-value"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn persists_desktop_preferences_between_processes() {
        let directory = std::env::temp_dir().join(format!(
            "game-translator-preferences-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("desktop-preferences.json");
        let preferences = super::DesktopPreferences {
            recent_project_path: Some("D:\\Games\\Moon".into()),
            source_language: super::LanguageInput {
                code: "ja".into(),
                name: "日语".into(),
            },
            target_language: super::LanguageInput {
                code: "zh-CN".into(),
                name: "简体中文".into(),
            },
        };

        super::write_desktop_preferences(&path, &preferences).unwrap();

        assert_eq!(super::read_desktop_preferences(&path).unwrap(), preferences);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn lists_patch_history_across_all_projects_newest_first() {
        let directory = std::env::temp_dir().join(format!(
            "game-translator-all-history-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let history = directory.join("patch-history.json");
        let first_project = directory.join("first");
        let second_project = directory.join("second");

        super::record_patch_export(
            &history,
            &first_project,
            &directory.join("one"),
            "zh-CN",
            1,
            10,
        )
        .unwrap();
        super::record_patch_export(
            &history,
            &second_project,
            &directory.join("two"),
            "zh-CN",
            1,
            20,
        )
        .unwrap();

        let entries = super::all_patch_history(&history).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].project_path, second_project.to_string_lossy());
        assert_eq!(entries[1].project_path, first_project.to_string_lossy());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn uninstalls_patch_by_restoring_backups_or_removing_unmodified_files() {
        let root = std::env::temp_dir().join(format!(
            "game-translator-uninstall-project-{}",
            std::process::id()
        ));
        let patch = std::env::temp_dir().join(format!(
            "game-translator-uninstall-patch-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&patch);
        std::fs::create_dir_all(root.join("renpy")).unwrap();
        std::fs::create_dir_all(root.join("game/tl/ja_JP")).unwrap();
        std::fs::write(root.join("Mayfly.py"), "").unwrap();
        let relative = PathBuf::from("game/tl/ja_JP/script.rpy");
        let original = "translate ja_JP start:\n    \"original\"\n";
        let translated = "translate ja_JP start:\n    \"translated\"\n";
        std::fs::write(root.join(&relative), original).unwrap();
        std::fs::create_dir_all(patch.join(relative.parent().unwrap())).unwrap();
        std::fs::write(patch.join(&relative), translated).unwrap();
        std::fs::create_dir_all(patch.join("backup").join(relative.parent().unwrap())).unwrap();
        std::fs::write(patch.join("backup").join(&relative), original).unwrap();
        let hash = super::file_sha256(&patch.join(&relative)).unwrap();
        std::fs::write(
            patch.join("patch-manifest.json"),
            format!(
                r#"{{"format_version":1,"files":[{{"relative_path":"game/tl/ja_JP/script.rpy","source_sha256":"","target_sha256":"{hash}"}}]}}"#
            ),
        )
        .unwrap();
        std::fs::write(root.join(&relative), translated).unwrap();

        let result = super::uninstall_patch_path(&root, &patch).unwrap();

        assert_eq!(result.restored_file_count, 1);
        assert_eq!(result.removed_file_count, 0);
        assert_eq!(
            std::fs::read_to_string(root.join(&relative)).unwrap(),
            original
        );
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(patch);
    }

    #[test]
    fn refuses_to_remove_a_patch_file_changed_after_installation() {
        let root = std::env::temp_dir().join(format!(
            "game-translator-uninstall-changed-project-{}",
            std::process::id()
        ));
        let patch = std::env::temp_dir().join(format!(
            "game-translator-uninstall-changed-patch-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&patch);
        std::fs::create_dir_all(root.join("renpy")).unwrap();
        std::fs::create_dir_all(root.join("game/tl/ja_JP")).unwrap();
        std::fs::write(root.join("Mayfly.py"), "").unwrap();
        let relative = PathBuf::from("game/tl/ja_JP/script.rpy");
        let translated = "translate ja_JP start:\n    \"translated\"\n";
        std::fs::create_dir_all(patch.join(relative.parent().unwrap())).unwrap();
        std::fs::write(patch.join(&relative), translated).unwrap();
        let hash = super::file_sha256(&patch.join(&relative)).unwrap();
        std::fs::write(
            patch.join("patch-manifest.json"),
            format!(
                r#"{{"format_version":1,"files":[{{"relative_path":"game/tl/ja_JP/script.rpy","source_sha256":"","target_sha256":"{hash}"}}]}}"#
            ),
        )
        .unwrap();
        std::fs::write(root.join(&relative), "changed by user").unwrap();

        let error = super::uninstall_patch_path(&root, &patch).unwrap_err();

        assert!(error.contains("已被修改"));
        assert_eq!(
            std::fs::read_to_string(root.join(&relative)).unwrap(),
            "changed by user"
        );
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(patch);
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
        assert!(output.join("game/tl/en_US/script.rpy").is_file());
        assert!(output.join("game/game_translator_language.rpy").is_file());
        let rendered = std::fs::read_to_string(output.join("game/tl/en_US/script.rpy")).unwrap();
        assert!(rendered.contains("译："));
        let _ = std::fs::remove_dir_all(output);
    }
}
