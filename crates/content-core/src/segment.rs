use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentKind {
    Dialogue,
    Choice,
    ScrollingText,
    DatabaseName,
    DatabaseDescription,
    LocalizedKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentContext {
    pub speaker: Option<String>,
    pub previous_text: Option<String>,
    pub next_text: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    pub id: String,
    pub source: String,
    pub source_file: PathBuf,
    /// Format-specific address of the source text within `source_file`.
    pub location: String,
    pub kind: SegmentKind,
    pub context: SegmentContext,
}
