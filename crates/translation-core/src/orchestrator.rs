use std::collections::HashMap;

use game_translator_provider_core::{
    TranslationInput, TranslationProvider, TranslationRequest, TranslationResponse,
};

use crate::{build_batches, retry::with_transient_retry, TranslationSegment};

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
}

pub struct TranslationOrchestrator<'a> {
    provider: &'a dyn TranslationProvider,
    model: String,
    source_language: String,
    target_language: String,
    maximum_batch_size: usize,
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
        }
    }

    #[must_use]
    pub fn run(
        &self,
        segments: &[TranslationSegment],
        cached: &HashMap<String, String>,
        control: RunControl,
    ) -> RunResult {
        let mut result = RunResult {
            status: RunStatus::Completed,
            translations: cached.clone(),
            failed_segment_ids: Vec::new(),
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
        for batch in build_batches(&pending, self.maximum_batch_size) {
            self.translate_with_split(&batch, &mut result);
        }
        if !result.failed_segment_ids.is_empty() {
            result.status = RunStatus::CompletedWithFailures;
        }
        result
    }

    fn translate_with_split(&self, batch: &[TranslationSegment], result: &mut RunResult) {
        match self.translate_batch(batch) {
            Ok(response) if response_matches(batch, &response) => {
                result.translations.extend(
                    response
                        .translations
                        .into_iter()
                        .map(|translation| (translation.id, translation.text)),
                );
            }
            Ok(_) | Err(_) if batch.len() > 1 => {
                let midpoint = batch.len() / 2;
                self.translate_with_split(&batch[..midpoint], result);
                self.translate_with_split(&batch[midpoint..], result);
            }
            Ok(_) | Err(_) => result
                .failed_segment_ids
                .extend(batch.iter().map(|segment| segment.id.clone())),
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
                .map(|segment| TranslationInput {
                    id: segment.id.clone(),
                    text: segment.source.clone(),
                })
                .collect(),
        };
        with_transient_retry(|| self.provider.translate(&request))
    }
}

fn response_matches(batch: &[TranslationSegment], response: &TranslationResponse) -> bool {
    batch.len() == response.translations.len()
        && batch
            .iter()
            .zip(&response.translations)
            .all(|(segment, translation)| segment.id == translation.id)
}
