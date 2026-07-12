use std::{
    collections::{HashMap, VecDeque},
    sync::{mpsc, Mutex},
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
}

pub struct TranslationOrchestrator<'a> {
    provider: &'a dyn TranslationProvider,
    model: String,
    source_language: String,
    target_language: String,
    maximum_batch_size: usize,
    maximum_batch_characters: usize,
    maximum_concurrency: usize,
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
        }
    }

    #[must_use]
    pub fn with_concurrency(mut self, maximum_concurrency: usize) -> Self {
        self.maximum_concurrency = maximum_concurrency.max(1);
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
        let (sender, receiver) = mpsc::channel();
        thread::scope(|scope| {
            for _ in 0..worker_count {
                let sender = sender.clone();
                let queue = &queue;
                scope.spawn(move || loop {
                    let batch = queue.lock().expect("batch queue poisoned").pop_front();
                    let Some(batch) = batch else { break };
                    if sender.send(BatchEvent::Started).is_err() {
                        break;
                    }
                    let started = Instant::now();
                    let mut outcome = BatchOutcome::default();
                    self.translate_with_split(&batch, &mut outcome);
                    outcome.elapsed_millis = started.elapsed().as_millis();
                    if sender.send(BatchEvent::Finished(outcome)).is_err() {
                        break;
                    }
                });
            }
            drop(sender);
            while let Ok(event) = receiver.recv() {
                match event {
                    BatchEvent::Started => result.active_requests += 1,
                    BatchEvent::Finished(outcome) => {
                        result.active_requests = result.active_requests.saturating_sub(1);
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
    Started,
    Finished(BatchOutcome),
}

fn response_matches(batch: &[TranslationSegment], response: &TranslationResponse) -> bool {
    batch.len() == response.translations.len()
        && batch
            .iter()
            .zip(&response.translations)
            .all(|(segment, translation)| segment.id == translation.id)
}
