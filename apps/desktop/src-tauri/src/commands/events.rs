use tauri::Emitter;

use super::dto::{ProgressMetrics, TranslationProgressEvent};

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_progress(
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
pub(super) fn emit_progress_with_metrics(
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
