use std::{collections::HashMap, path::Path, path::PathBuf, time::Instant};

use game_translator_app_core::{
    detect_game, engine_name, extract_game, CredentialStore, PatchManifest, PatchPlan,
    WindowsCredentialStore,
};
use game_translator_engine_core::{EngineKind, Segment};
use game_translator_project_store::{CacheEntry, ProjectStore};
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
use sha2::{Digest, Sha256};
use tauri::{Emitter, Manager};

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
    #[serde(default)]
    performance: Option<String>,
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
    concurrency: usize,
    throughput: f64,
    eta_seconds: usize,
}

#[derive(Clone, Copy, Default)]
struct ProgressMetrics {
    concurrency: usize,
    throughput: f64,
    eta_seconds: usize,
}

struct TranslationExecution<'a> {
    model: &'a str,
    source_language: &'a str,
    target_language: &'a str,
    cache_scope: &'a str,
    store: Option<&'a ProjectStore>,
    batch_size: usize,
    character_budget: usize,
    initial_concurrency: usize,
    concurrency: usize,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallCommandInput {
    project_path: String,
    patch_path: String,
    target_language: LanguageInput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallResult {
    installed_path: String,
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
        performance: _,
    } = provider;
    if kind == "openai" {
        let mut credentials = WindowsCredentialStore::new("GameTranslator");
        if let Some(secret) = api_key.as_deref().filter(|secret| !secret.is_empty()) {
            credentials
                .set("default-provider", secret)
                .map_err(|error| error.to_string())?;
        } else if credentials
            .get("default-provider")
            .map_err(|error| error.to_string())?
            .is_none()
        {
            return Err("OpenAI-compatible Provider 需要 API Key".to_owned());
        }
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
    let provider: Box<dyn TranslationProvider> = if kind == "ollama" {
        Box::new(OllamaProvider::new(&base_url))
    } else {
        let credentials = WindowsCredentialStore::new("GameTranslator");
        let api_key = credentials
            .get("default-provider")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "尚未保存 API Key".to_owned())?;
        let provider = OpenAiCompatibleProvider::new(&base_url, api_key);
        if base_url.contains("deepseek") || model.starts_with("deepseek-") {
            Box::new(provider.with_user_id("game-translator-desktop"))
        } else {
            Box::new(provider)
        }
    };
    let (batch_size, character_budget, initial_concurrency, concurrency) =
        performance_settings(&kind, performance.as_deref().unwrap_or("balanced"));
    let database_path = app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("translations.sqlite3");
    let store = ProjectStore::open(&database_path).map_err(|error| error.to_string())?;
    let source_prompt = language_prompt(&source_language);
    let target_prompt = language_prompt(&target_language);
    let cache_scope = format!("v2\0{kind}\0{base_url}");
    translate_path_with_provider_and_progress(
        Path::new(&project_path),
        provider.as_ref(),
        &TranslationExecution {
            model: &model,
            source_language: &source_prompt,
            target_language: &target_prompt,
            cache_scope: &cache_scope,
            store: Some(&store),
            batch_size,
            character_budget,
            initial_concurrency,
            concurrency,
        },
        |phase, completed, total, failed, warnings, blocking, message, metrics| {
            emit_progress_with_metrics(
                app, &run_id, phase, completed, total, failed, warnings, blocking, message, metrics,
            );
        },
    )
}

fn performance_settings(kind: &str, mode: &str) -> (usize, usize, usize, usize) {
    match (kind, mode) {
        ("ollama", "fast") => (48, 20_000, 2, 2),
        ("ollama", _) => (32, 16_000, 1, 1),
        (_, "stable") => (32, 16_000, 4, 16),
        (_, "fast") => (16, 8_000, 16, 128),
        _ => (24, 12_000, 8, 64),
    }
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
    emit_progress_with_metrics(
        app,
        run_id,
        phase,
        completed,
        total,
        failed,
        warning_findings,
        blocking_findings,
        message,
        ProgressMetrics::default(),
    );
}

#[allow(clippy::too_many_arguments)]
fn emit_progress_with_metrics(
    app: &tauri::AppHandle,
    run_id: &str,
    phase: &str,
    completed: usize,
    total: usize,
    failed: usize,
    warning_findings: usize,
    blocking_findings: usize,
    message: &str,
    metrics: ProgressMetrics,
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
            concurrency: metrics.concurrency,
            throughput: metrics.throughput,
            eta_seconds: metrics.eta_seconds,
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
        &TranslationExecution {
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
    let run = orchestrator.run_with_progress(
        &translation_segments,
        &cached,
        RunControl::Running,
        |current| {
            let completed = current.translations.len() + current.failed_segment_ids.len();
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
    let (items, warning_findings, blocking_findings) = restore_and_check(
        segments,
        &run,
        &protected,
        &cached,
        &fingerprints,
        execution.store,
        &mut progress,
    )?;

    progress(
        "completed",
        total,
        total,
        run.failed_segment_ids.len(),
        warning_findings,
        blocking_findings,
        "任务完成，可以校对或导出",
        ProgressMetrics::default(),
    );

    Ok(TranslationRunResult {
        items,
        warning_findings,
        blocking_findings,
        failed_segment_ids: run.failed_segment_ids,
    })
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
) -> Result<(Vec<TranslationItem>, usize, usize), String>
where
    F: FnMut(&str, usize, usize, usize, usize, usize, &str, ProgressMetrics),
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

    Ok((items, warning_findings, blocking_findings))
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
fn export_translation_patch(input: ExportCommandInput) -> Result<ExportResult, String> {
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

#[tauri::command]
fn install_translation_patch(input: InstallCommandInput) -> Result<InstallResult, String> {
    let InstallCommandInput {
        project_path,
        patch_path,
        target_language,
    } = input;
    install_patch_path(
        Path::new(&project_path),
        Path::new(&patch_path),
        &target_language.code,
    )
}

fn install_patch_path(
    project_path: &Path,
    patch_path: &Path,
    target_language: &str,
) -> Result<InstallResult, String> {
    let project = detect_game(project_path).map_err(|error| error.to_string())?;
    if project.engine != EngineKind::RenPy {
        return Err("当前自动安装仅支持 Ren'Py；其他引擎请使用导出的独立补丁目录".into());
    }
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
    let language = game_translator_engine_renpy::language_identifier(target_language);
    Ok(InstallResult {
        installed_path: project_path
            .join("game/tl")
            .join(language)
            .to_string_lossy()
            .into_owned(),
        file_count: manifest.files.len(),
    })
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
            translate_project,
            export_translation_patch,
            install_translation_patch
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GameTranslator");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn desktop_uses_the_core_product_name() {
        assert_eq!(game_translator_app_core::product_name(), "GameTranslator");
    }

    #[test]
    fn deepseek_fast_mode_uses_small_batches_and_adaptive_headroom() {
        assert_eq!(
            super::performance_settings("openai", "fast"),
            (16, 8_000, 16, 128)
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
        let segments = super::extract_game(&project).unwrap();
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
