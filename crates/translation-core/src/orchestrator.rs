use std::{
    collections::{HashMap, VecDeque},
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Instant,
};

use game_translator_provider_core::{
    TranslationInput, TranslationProvider, TranslationRequest, TranslationResponse,
};

use crate::{build_batches_with_budget, retry::with_transient_retry, TranslationSegment};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunControl {
    Running,
    Paused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStatus {
    Completed,
    CompletedWithFailures,
    Paused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunResult {
    pub status: RunStatus,
    pub translations: HashMap<String, String>,
    pub failed_segment_ids: Vec<String>,
    pub active_requests: usize,
    pub completed_batches: usize,
    pub total_batches: usize,
    pub last_batch_millis: u128,
    pub concurrency_limit: usize,
    pub throttle_events: usize,
}

pub struct TranslationOrchestrator<'a> {
    provider: &'a dyn TranslationProvider,
    model: String,
    source_language: String,
    target_language: String,
    maximum_batch_size: usize,
    maximum_batch_characters: usize,
    maximum_concurrency: usize,
    initial_concurrency: usize,
}

impl<'a> TranslationOrchestrator<'a> {
    #[must_use]
    pub fn new(
        provider: &'a dyn TranslationProvider,
        model: impl Into<String>,
        source_language: impl Into<String>,
        target_language: impl Into<String>,
        maximum_batch_size: usize,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            source_language: source_language.into(),
            target_language: target_language.into(),
            maximum_batch_size: maximum_batch_size.max(1),
            maximum_batch_characters: 24_000,
            maximum_concurrency: 1,
            initial_concurrency: 1,
        }
    }

    #[must_use]
    pub fn with_concurrency(mut self, maximum_concurrency: usize) -> Self {
        self.maximum_concurrency = maximum_concurrency.max(1);
        self.initial_concurrency = self.maximum_concurrency;
        self
    }

    #[must_use]
    pub fn with_adaptive_concurrency(
        mut self,
        initial_concurrency: usize,
        maximum_concurrency: usize,
    ) -> Self {
        self.maximum_concurrency = maximum_concurrency.max(1);
        self.initial_concurrency = initial_concurrency.clamp(1, self.maximum_concurrency);
        self
    }

    #[must_use]
    pub fn with_batch_character_budget(mut self, maximum_characters: usize) -> Self {
        self.maximum_batch_characters = maximum_characters.max(1);
        self
    }

    #[must_use]
    pub fn run(
        &self,
        segments: &[TranslationSegment],
        cached: &HashMap<String, String>,
        control: RunControl,
    ) -> RunResult {
        self.run_with_progress(segments, cached, control, |_| {})
    }

    #[must_use]
    pub fn run_with_progress<F>(
        &self,
        segments: &[TranslationSegment],
        cached: &HashMap<String, String>,
        control: RunControl,
        mut on_progress: F,
    ) -> RunResult
    where
        F: FnMut(&RunResult),
    {
        let mut result = RunResult {
            status: RunStatus::Completed,
            translations: cached.clone(),
            failed_segment_ids: Vec::new(),
            active_requests: 0,
            completed_batches: 0,
            total_batches: 0,
            last_batch_millis: 0,
            concurrency_limit: self.initial_concurrency,
            throttle_events: 0,
        };
        if control == RunControl::Paused {
            result.status = RunStatus::Paused;
            return result;
        }

        let pending = segments
            .iter()
            .filter(|segment| !cached.contains_key(&segment.id))
            .cloned()
            .collect::<Vec<_>>();
        let batches = build_batches_with_budget(
            &pending,
            self.maximum_batch_size,
            self.maximum_batch_characters,
        );
        self.run_batches(batches, &mut result, &mut on_progress);
        if !result.failed_segment_ids.is_empty() {
            result.status = RunStatus::CompletedWithFailures;
        }
        result
    }

    fn run_batches<F>(
        &self,
        batches: Vec<Vec<TranslationSegment>>,
        result: &mut RunResult,
        on_progress: &mut F,
    ) where
        F: FnMut(&RunResult),
    {
        if batches.is_empty() {
            return;
        }
        result.total_batches = batches.len();
        let worker_count = self.maximum_concurrency.min(batches.len());
        let queue = Mutex::new(VecDeque::from(batches));
        let limiter = Arc::new(AdaptiveLimiter::new(
            self.initial_concurrency.min(worker_count),
            worker_count,
        ));
        let (sender, receiver) = mpsc::channel();
        thread::scope(|scope| {
            for _ in 0..worker_count {
                let sender = sender.clone();
                let queue = &queue;
                let limiter = Arc::clone(&limiter);
                scope.spawn(move || loop {
                    let batch = queue.lock().expect("batch queue poisoned").pop_front();
                    let Some(batch) = batch else { break };
                    let concurrency_limit = limiter.acquire();
                    if sender
                        .send(BatchEvent::Started { concurrency_limit })
                        .is_err()
                    {
                        limiter.release(false);
                        break;
                    }
                    let started = Instant::now();
                    let mut outcome = BatchOutcome::default();
                    self.translate_with_split(&batch, &mut outcome);
                    outcome.elapsed_millis = started.elapsed().as_millis();
                    let state = limiter.release(outcome.failed_segment_ids.is_empty());
                    if sender
                        .send(BatchEvent::Finished { outcome, state })
                        .is_err()
                    {
                        break;
                    }
                });
            }
            drop(sender);
            while let Ok(event) = receiver.recv() {
                match event {
                    BatchEvent::Started { concurrency_limit } => {
                        result.active_requests += 1;
                        result.concurrency_limit = concurrency_limit;
                    }
                    BatchEvent::Finished { outcome, state } => {
                        result.active_requests = result.active_requests.saturating_sub(1);
                        result.concurrency_limit = state.target;
                        result.throttle_events = state.throttle_events;
                        result.completed_batches += 1;
                        result.last_batch_millis = outcome.elapsed_millis;
                        result.translations.extend(outcome.translations);
                        result.failed_segment_ids.extend(outcome.failed_segment_ids);
                    }
                }
                on_progress(result);
            }
        });
    }

    fn translate_with_split(&self, batch: &[TranslationSegment], outcome: &mut BatchOutcome) {
        match self.translate_batch(batch) {
            Ok(response) if response_matches(batch, &response) => {
                outcome.translations.extend(
                    response
                        .translations
                        .into_iter()
                        .map(|translation| (translation.id, translation.text)),
                );
            }
            Ok(_) | Err(_) if batch.len() > 1 => {
                let midpoint = batch.len() / 2;
                self.translate_with_split(&batch[..midpoint], outcome);
                self.translate_with_split(&batch[midpoint..], outcome);
            }
            Ok(_) | Err(_) => {
                outcome
                    .failed_segment_ids
                    .extend(batch.iter().map(|segment| segment.id.clone()));
            }
        }
    }

    fn translate_batch(
        &self,
        batch: &[TranslationSegment],
    ) -> Result<TranslationResponse, game_translator_provider_core::ProviderError> {
        let request = TranslationRequest {
            model: self.model.clone(),
            source_language: self.source_language.clone(),
            target_language: self.target_language.clone(),
            segments: batch
                .iter()
                .enumerate()
                .map(|(index, segment)| TranslationInput {
                    id: index.to_string(),
                    text: segment.source.clone(),
                })
                .collect(),
        };
        let mut response = with_transient_retry(|| self.provider.translate(&request))?;
        if response.translations.len() != batch.len()
            || response
                .translations
                .iter()
                .enumerate()
                .any(|(index, translation)| translation.id != index.to_string())
        {
            return Err(
                game_translator_provider_core::ProviderError::InvalidResponse(
                    "batch-local translation ids are missing or out of order".into(),
                ),
            );
        }
        for (translation, segment) in response.translations.iter_mut().zip(batch) {
            translation.id.clone_from(&segment.id);
        }
        Ok(response)
    }
}

#[derive(Default)]
struct BatchOutcome {
    translations: HashMap<String, String>,
    failed_segment_ids: Vec<String>,
    elapsed_millis: u128,
}

enum BatchEvent {
    Started {
        concurrency_limit: usize,
    },
    Finished {
        outcome: BatchOutcome,
        state: LimiterSnapshot,
    },
}

struct AdaptiveLimiter {
    state: Mutex<LimiterState>,
    changed: Condvar,
    maximum: usize,
}

struct LimiterState {
    active: usize,
    target: usize,
    successful_batches: usize,
    throttle_events: usize,
}

#[derive(Clone, Copy)]
struct LimiterSnapshot {
    target: usize,
    throttle_events: usize,
}

impl AdaptiveLimiter {
    fn new(initial: usize, maximum: usize) -> Self {
        Self {
            state: Mutex::new(LimiterState {
                active: 0,
                target: initial,
                successful_batches: 0,
                throttle_events: 0,
            }),
            changed: Condvar::new(),
            maximum,
        }
    }

    fn acquire(&self) -> usize {
        let mut state = self.state.lock().expect("adaptive limiter poisoned");
        while state.active >= state.target {
            state = self.changed.wait(state).expect("adaptive limiter poisoned");
        }
        state.active += 1;
        state.target
    }

    fn release(&self, success: bool) -> LimiterSnapshot {
        let mut state = self.state.lock().expect("adaptive limiter poisoned");
        state.active = state.active.saturating_sub(1);
        if success {
            state.successful_batches += 1;
            if state.successful_batches % 4 == 0 && state.target < self.maximum {
                state.target += 1;
            }
        } else {
            state.target = (state.target / 2).max(1);
            state.throttle_events += 1;
            state.successful_batches = 0;
        }
        let snapshot = LimiterSnapshot {
            target: state.target,
            throttle_events: state.throttle_events,
        };
        self.changed.notify_all();
        snapshot
    }
}

fn response_matches(batch: &[TranslationSegment], response: &TranslationResponse) -> bool {
    batch.len() == response.translations.len()
        && batch
            .iter()
            .zip(&response.translations)
            .all(|(segment, translation)| segment.id == translation.id)
}
