use std::{error::Error, fmt, path::Path, path::PathBuf};

use crate::{DetectedProject, Segment};

#[derive(Debug, Eq, PartialEq)]
pub enum EngineError {
    UnsupportedProject,
    MissingRequiredFile(PathBuf),
    InvalidData { path: PathBuf, message: String },
    Io { path: PathBuf, message: String },
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProject => formatter.write_str("unsupported game project"),
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

impl Error for EngineError {}

pub trait EngineAdapter {
    /// Identifies a supported game project without modifying it.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] when the project is unsupported, incomplete, or unreadable.
    fn detect(&self, root: &Path) -> Result<DetectedProject, EngineError>;

    /// Extracts translatable segments from a detected project.
    ///
    /// # Errors
    ///
    /// Returns an [`EngineError`] when a required source file cannot be read or parsed.
    fn extract(&self, project: &DetectedProject) -> Result<Vec<Segment>, EngineError>;
}
