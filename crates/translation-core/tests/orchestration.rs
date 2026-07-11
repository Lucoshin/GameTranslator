use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, atomic::Ordering, Mutex},
};

use game_translator_provider_core::{
    ProviderError, TranslationOutput, TranslationProvider, TranslationRequest, TranslationResponse,
};
use game_translator_translation_core::{
    build_batches, RunControl, RunStatus, TranslationOrchestrator, TranslationSegment,
};

struct FakeProvider {
    calls: Mutex<Vec<Vec<String>>>,
    fail_large_batches: bool,
}

struct RateLimitedOnceProvider {
    calls: AtomicUsize,
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
