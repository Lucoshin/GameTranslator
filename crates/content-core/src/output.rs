use std::{collections::HashMap, hash::BuildHasher, path::Path, path::PathBuf};

use crate::{ContentError, ContentSource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputCapability {
    Export,
    Install,
    Uninstall,
}

pub struct ExportRequest<'a, S: BuildHasher> {
    pub source: &'a ContentSource,
    pub translations: &'a HashMap<String, String, S>,
    pub output_root: &'a Path,
    pub target_language: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportResult {
    pub output_root: PathBuf,
    pub files: Vec<PathBuf>,
}

pub trait ContentOutputAdapter {
    fn format_id(&self) -> &'static str;

    fn capabilities(&self) -> &'static [OutputCapability];

    /// # Errors
    ///
    /// Returns an error when the source is incompatible, a translation cannot be rendered, or
    /// the output directory cannot be written.
    fn export<S: BuildHasher>(
        &self,
        request: &ExportRequest<'_, S>,
    ) -> Result<ExportResult, ContentError>;
}
