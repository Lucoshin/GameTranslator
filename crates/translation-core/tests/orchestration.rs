use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, atomic::Ordering, Mutex},
    thread,
    time::Duration,
};

use game_translator_provider_core::{
    ProviderError, TranslationOutput, TranslationProvider, TranslationRequest, TranslationResponse,
};
use game_translator_translation_core::{
    build_batches, build_batches_with_budget, RunControl, RunStatus, TranslationOrchestrator,
    TranslationSegment,
};

struct FakeProvider {
    calls: Mutex<Vec<Vec<String>>>,
    fail_large_batches: bool,
}

struct RateLimitedOnceProvider {
    calls: AtomicUsize,
}

struct ConcurrentProvider {
    active: AtomicUsize,
    maximum: AtomicUsize,
}

impl TranslationProvider for ConcurrentProvider {
    fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, ProviderError> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(30));
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(TranslationResponse {
            translations: request
                .segments
                .iter()
                .map(|segment| TranslationOutput {
                    id: segment.id.clone(),
                    text: segment.text.clone(),
                })
                .collect(),
        })
    }
}

impl TranslationProvider for RateLimitedOnceProvider {
    fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, ProviderError> {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            return Err(ProviderError::RateLimited);
        }
        Ok(TranslationResponse {
            translations: request
                .segments
                .iter()
                .map(|segment| TranslationOutput {
                    id: segment.id.clone(),
                    text: "重试成功".into(),
                })
                .collect(),
        })
    }
}

impl TranslationProvider for FakeProvider {
    fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, ProviderError> {
        self.calls.lock().unwrap().push(
            request
                .segments
                .iter()
                .map(|segment| segment.id.clone())
                .collect(),
        );
        if self.fail_large_batches && request.segments.len() > 1 {
            return Err(ProviderError::InvalidResponse("batch too large".into()));
        }
        Ok(TranslationResponse {
            translations: request
                .segments
                .iter()
                .map(|segment| TranslationOutput {
                    id: segment.id.clone(),
                    text: format!("译：{}", segment.text),
                })
                .collect(),
        })
    }
}

fn segments() -> Vec<TranslationSegment> {
    vec![
        TranslationSegment::new("a", "map-1", "A"),
        TranslationSegment::new("b", "map-1", "B"),
        TranslationSegment::new("c", "map-2", "C"),
    ]
}

#[test]
fn groups_segments_by_scene_and_batch_limit() {
    let batches = build_batches(&segments(), 2);

    assert_eq!(batches.len(), 2);
    assert_eq!(
        batches[0]
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(batches[1][0].id, "c");
}

#[test]
fn splits_batches_when_the_character_budget_is_reached() {
    let segments = vec![
        TranslationSegment::new("a", "scene", "12345"),
        TranslationSegment::new("b", "scene", "67890"),
        TranslationSegment::new("c", "scene", "x"),
    ];

    let batches = build_batches_with_budget(&segments, 64, 6);

    assert_eq!(batches.iter().map(Vec::len).collect::<Vec<_>>(), vec![1, 2]);
    assert!(batches.iter().all(|batch| {
        batch
            .iter()
            .map(|segment| segment.source.chars().count())
            .sum::<usize>()
            <= 6
    }));
}

#[test]
fn skips_cached_segments_and_returns_all_results() {
    let provider = FakeProvider {
        calls: Mutex::new(Vec::new()),
        fail_large_batches: false,
    };
    let cached = HashMap::from([("a".to_string(), "缓存A".to_string())]);
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 2);

    let result = orchestrator.run(&segments(), &cached, RunControl::Running);

    assert_eq!(result.status, RunStatus::Completed);
    assert_eq!(result.translations.get("a").unwrap(), "缓存A");
    assert_eq!(provider.calls.lock().unwrap().len(), 2);
}

#[test]
fn paused_run_does_not_call_the_provider() {
    let provider = FakeProvider {
        calls: Mutex::new(Vec::new()),
        fail_large_batches: false,
    };
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 2);

    let result = orchestrator.run(&segments(), &HashMap::new(), RunControl::Paused);

    assert_eq!(result.status, RunStatus::Paused);
    assert!(provider.calls.lock().unwrap().is_empty());
}

#[test]
fn splits_a_failed_batch_before_marking_segments_failed() {
    let provider = FakeProvider {
        calls: Mutex::new(Vec::new()),
        fail_large_batches: true,
    };
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 2);

    let result = orchestrator.run(&segments()[..2], &HashMap::new(), RunControl::Running);

    assert_eq!(result.status, RunStatus::Completed);
    assert!(result.failed_segment_ids.is_empty());
    assert_eq!(provider.calls.lock().unwrap().len(), 3);
}

#[test]
fn retries_a_rate_limited_request_before_failing_it() {
    let provider = RateLimitedOnceProvider {
        calls: AtomicUsize::new(0),
    };
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 1);

    let result = orchestrator.run(&segments()[..1], &HashMap::new(), RunControl::Running);

    assert_eq!(result.status, RunStatus::Completed);
    assert_eq!(result.translations.get("a").unwrap(), "重试成功");
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn reports_real_progress_after_each_completed_batch() {
    let provider = FakeProvider {
        calls: Mutex::new(Vec::new()),
        fail_large_batches: false,
    };
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 2);
    let segments = segments();
    let mut completed = Vec::new();

    let result = orchestrator.run_with_progress(
        &segments,
        &HashMap::new(),
        RunControl::Running,
        |progress| {
            if progress.completed_batches > completed.len() {
                completed.push(progress.translations.len());
            }
        },
    );

    assert_eq!(result.translations.len(), 3);
    assert_eq!(completed, vec![2, 3]);
}

#[test]
fn sends_short_batch_local_ids_to_the_provider() {
    let provider = FakeProvider {
        calls: Mutex::new(Vec::new()),
        fail_large_batches: false,
    };
    let orchestrator = TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 2);

    let result = orchestrator.run(&segments()[..2], &HashMap::new(), RunControl::Running);

    assert_eq!(result.translations.get("a").unwrap(), "译：A");
    assert_eq!(provider.calls.lock().unwrap()[0], ["0", "1"]);
}

#[test]
fn runs_independent_batches_with_bounded_concurrency() {
    let provider = ConcurrentProvider {
        active: AtomicUsize::new(0),
        maximum: AtomicUsize::new(0),
    };
    let orchestrator =
        TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 1).with_concurrency(3);
    let segments = (0..6)
        .map(|index| TranslationSegment::new(index.to_string(), index.to_string(), "text"))
        .collect::<Vec<_>>();

    let result = orchestrator.run(&segments, &HashMap::new(), RunControl::Running);

    assert_eq!(result.translations.len(), 6);
    assert_eq!(provider.maximum.load(Ordering::SeqCst), 3);
}

#[test]
fn reports_in_flight_requests_before_the_first_batch_finishes() {
    let provider = ConcurrentProvider {
        active: AtomicUsize::new(0),
        maximum: AtomicUsize::new(0),
    };
    let orchestrator =
        TranslationOrchestrator::new(&provider, "model", "auto", "zh-CN", 1).with_concurrency(3);
    let segments = (0..6)
        .map(|index| TranslationSegment::new(index.to_string(), index.to_string(), "text"))
        .collect::<Vec<_>>();
    let mut snapshots = Vec::new();

    let _ = orchestrator.run_with_progress(
        &segments,
        &HashMap::new(),
        RunControl::Running,
        |progress| {
            snapshots.push((
                progress.active_requests,
                progress.completed_batches,
                progress.total_batches,
            ));
        },
    );

    assert!(snapshots.contains(&(3, 0, 6)));
    assert_eq!(snapshots.last(), Some(&(0, 6, 6)));
}
