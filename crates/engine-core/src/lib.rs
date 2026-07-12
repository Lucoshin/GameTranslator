mod adapter;
mod project;

pub use adapter::{EngineAdapter, EngineError};
pub use game_translator_content_core::{Segment, SegmentContext, SegmentKind};
pub use project::{DetectedProject, EngineKind};
