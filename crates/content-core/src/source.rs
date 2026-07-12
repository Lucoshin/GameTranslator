use std::{error::Error, fmt, path::Path, path::PathBuf};

use crate::{OutputCapability, Segment};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentCategory {
    Game,
    GameMod,
    Document,
    Subtitle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSource {
    pub root: PathBuf,
    pub format_id: &'static str,
    pub display_name: String,
    pub source_id: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ContentError {
    UnsupportedSource,
    MissingRequiredFile(PathBuf),
    InvalidData { path: PathBuf, message: String },
    Io { path: PathBuf, message: String },
}

impl fmt::Display for ContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSource => formatter.write_str("unsupported content source"),
            Self::MissingRequiredFile(path) => {
                write!(formatter, "missing required file: {}", path.display())
            }
            Self::InvalidData { path, message } => {
                write!(formatter, "invalid data in {}: {message}", path.display())
            }
            Self::Io { path, message } => {
                write!(formatter, "failed to read {}: {message}", path.display())
            }
        }
    }
}

impl Error for ContentError {}

pub trait ContentSourceAdapter {
    fn format_id(&self) -> &'static str;

    fn category(&self) -> ContentCategory;

    fn output_capabilities(&self) -> &'static [OutputCapability];

    /// # Errors
    ///
    /// Returns an error when the selected root is not a supported source or cannot be read.
    fn detect(&self, root: &Path) -> Result<ContentSource, ContentError>;

    /// # Errors
    ///
    /// Returns an error when source files cannot be read or parsed.
    fn extract(&self, source: &ContentSource) -> Result<Vec<Segment>, ContentError>;
}
