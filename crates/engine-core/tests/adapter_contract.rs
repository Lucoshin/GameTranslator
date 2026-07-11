use std::path::Path;

use game_translator_engine_core::{DetectedProject, EngineAdapter, EngineError, Segment};

struct EmptyAdapter;

impl EngineAdapter for EmptyAdapter {
    fn detect(&self, _root: &Path) -> Result<DetectedProject, EngineError> {
        Err(EngineError::UnsupportedProject)
    }

    fn extract(&self, _project: &DetectedProject) -> Result<Vec<Segment>, EngineError> {
        Ok(Vec::new())
    }
}

#[test]
fn adapter_contract_uses_explicit_results() {
    let adapter = EmptyAdapter;
    let error = adapter.detect(Path::new("unsupported")).unwrap_err();

    assert_eq!(error, EngineError::UnsupportedProject);
}
