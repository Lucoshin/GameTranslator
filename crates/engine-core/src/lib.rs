mod adapter;
mod project;
mod segment;

pub use adapter::{EngineAdapter, EngineError};
pub use project::{DetectedProject, EngineKind};
pub use segment::{Segment, SegmentContext, SegmentKind};
