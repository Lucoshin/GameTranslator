use game_translator_project_store::ProjectStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScanResult {
    pub project_path: String,
    pub project_name: String,
    pub engine: String,
    pub segment_count: usize,
    pub preview_items: Vec<TranslationItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProviderInput {
    pub kind: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    #[serde(default)]
    pub performance: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProviderMetadata {
    pub kind: String,
    pub base_url: String,
    pub model: String,
    pub performance: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslateCommandInput {
    pub run_id: String,
    pub project_path: String,
    pub provider: ProviderInput,
    pub source_language: LanguageInput,
    pub target_language: LanguageInput,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslationProgressEvent {
    pub run_id: String,
    pub phase: String,
    pub completed: usize,
    pub total: usize,
    pub failed: usize,
    pub warning_findings: usize,
    pub blocking_findings: usize,
    pub message: String,
    pub concurrency: usize,
    pub throughput: f64,
    pub eta_seconds: usize,
}

#[derive(Clone, Copy, Default)]
pub(super) struct ProgressMetrics {
    pub concurrency: usize,
    pub throughput: f64,
    pub eta_seconds: usize,
}

pub(super) struct TranslationExecution<'a> {
    pub task_id: &'a str,
    pub model: &'a str,
    pub source_language: &'a str,
    pub target_language: &'a str,
    pub cache_scope: &'a str,
    pub store: Option<&'a ProjectStore>,
    pub batch_size: usize,
    pub character_budget: usize,
    pub initial_concurrency: usize,
    pub concurrency: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LanguageInput {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DesktopPreferences {
    pub recent_project_path: Option<String>,
    pub source_language: LanguageInput,
    pub target_language: LanguageInput,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            recent_project_path: None,
            source_language: LanguageInput {
                code: "auto".into(),
                name: "自动检测".into(),
            },
            target_language: LanguageInput {
                code: "zh-CN".into(),
                name: "简体中文".into(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResumableTask {
    pub id: String,
    pub project_path: String,
    pub state: String,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub updated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslationItem {
    pub id: String,
    pub source: String,
    pub target: String,
    pub speaker: Option<String>,
    pub source_file: String,
    pub qa: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslationRunResult {
    pub items: Vec<TranslationItem>,
    pub warning_findings: usize,
    pub blocking_findings: usize,
    pub failed_segment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExportCommandInput {
    pub project_path: String,
    pub items: Vec<TranslationItem>,
    pub target_language: LanguageInput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExportResult {
    pub output_path: String,
    pub file_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstallCommandInput {
    pub project_path: String,
    pub patch_path: String,
    pub target_language: LanguageInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UninstallCommandInput {
    pub project_path: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeletePatchHistoryInput {
    pub project_path: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstallResult {
    pub installed_path: String,
    pub file_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PatchHistoryEntry {
    pub id: String,
    pub project_path: String,
    pub patch_path: String,
    pub target_language: String,
    pub file_count: usize,
    pub exported_at_unix_ms: u64,
    pub installed_at_unix_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UninstallResult {
    pub restored_file_count: usize,
    pub removed_file_count: usize,
}
