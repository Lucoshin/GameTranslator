use std::{collections::HashMap, hash::BuildHasher, path::Path};

use game_translator_content_core::{ContentError, ContentSource, ExportResult, Segment};

use crate::AdapterRegistry;

/// Detects a registered content source without modifying its files.
///
/// # Errors
///
/// Returns an error when no registered adapter recognizes the selected directory.
pub fn detect_content(root: &Path) -> Result<ContentSource, ContentError> {
    AdapterRegistry::default().detect(root)
}

/// Extracts text segments using the adapter identified by a content source.
///
/// # Errors
///
/// Returns an error when the source format is not registered or cannot be read.
pub fn extract_content(source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
    AdapterRegistry::default().extract(source)
}

/// Exports content through the output adapter for its source format.
///
/// # Errors
///
/// Returns an error when the source format has no registered output adapter or output fails.
pub fn export_content<S: BuildHasher>(
    source: &ContentSource,
    translations: &HashMap<String, String, S>,
    output_root: &Path,
    target_language: &str,
) -> Result<ExportResult, ContentError> {
    AdapterRegistry::default().export(source, translations, output_root, target_language)
}
