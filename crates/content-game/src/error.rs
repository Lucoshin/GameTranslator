use game_translator_content_core::ContentError;
use game_translator_engine_core::EngineError;

pub(crate) fn map_engine_error(error: EngineError) -> ContentError {
    match error {
        EngineError::UnsupportedProject => ContentError::UnsupportedSource,
        EngineError::MissingRequiredFile(path) => ContentError::MissingRequiredFile(path),
        EngineError::InvalidData { path, message } => ContentError::InvalidData { path, message },
        EngineError::Io { path, message } => ContentError::Io { path, message },
    }
}
