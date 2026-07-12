use std::{
    collections::HashMap, error::Error, fmt, fs, hash::BuildHasher, path::Path, path::PathBuf,
};

use game_translator_engine_core::{DetectedProject, EngineKind};
use game_translator_qa_core::{QaFinding, QaSeverity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PatchFile {
    pub relative_path: PathBuf,
    pub source_sha256: String,
    pub target_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PatchManifest {
    pub format_version: u32,
    pub files: Vec<PatchFile>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PatchError {
    BlockingQualityFindings,
    SourceChanged(PathBuf),
    Engine(String),
    Io { path: PathBuf, message: String },
}

impl fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockingQualityFindings => {
                formatter.write_str("blocking quality findings prevent export")
            }
            Self::SourceChanged(path) => {
                write!(formatter, "source file changed: {}", path.display())
            }
            Self::Engine(message) => formatter.write_str(message),
            Self::Io { path, message } => {
                write!(formatter, "failed to access {}: {message}", path.display())
            }
        }
    }
}

impl Error for PatchError {}

pub struct PatchPlan {
    project: DetectedProject,
    source_hashes: HashMap<PathBuf, String>,
}

impl PatchPlan {
    /// Captures hashes for every source file containing an extracted segment.
    ///
    /// # Errors
    ///
    /// Returns [`PatchError`] when extraction or source hashing fails.
    pub fn capture(project: DetectedProject) -> Result<Self, PatchError> {
        let segments =
            crate::extract_game(&project).map_err(|error| PatchError::Engine(error.to_string()))?;
        let mut source_hashes = HashMap::new();
        for segment in segments {
            if !source_hashes.contains_key(&segment.source_file) {
                source_hashes.insert(
                    segment.source_file.clone(),
                    hash_file(&segment.source_file)?,
                );
            }
        }
        Ok(Self {
            project,
            source_hashes,
        })
    }

    /// Verifies source files, writes translated JSON, and emits a hash manifest.
    ///
    /// # Errors
    ///
    /// Returns [`PatchError`] for blocking QA, changed sources, invalid engine data, or I/O failures.
    pub fn export<S: BuildHasher>(
        &self,
        translations: &HashMap<String, String, S>,
        findings: &[QaFinding],
        output_root: &Path,
    ) -> Result<PatchManifest, PatchError> {
        self.export_for_language(translations, findings, output_root, "zh-CN")
    }

    /// Exports an engine-native patch for the requested target language.
    ///
    /// # Errors
    /// Returns [`PatchError`] for blocking QA, changed sources, or engine write failures.
    pub fn export_for_language<S: BuildHasher>(
        &self,
        translations: &HashMap<String, String, S>,
        findings: &[QaFinding],
        output_root: &Path,
        language: &str,
    ) -> Result<PatchManifest, PatchError> {
        if findings
            .iter()
            .any(|finding| finding.severity == QaSeverity::Blocking)
        {
            return Err(PatchError::BlockingQualityFindings);
        }
        for (path, captured_hash) in &self.source_hashes {
            if &hash_file(path)? != captured_hash {
                return Err(PatchError::SourceChanged(path.clone()));
            }
        }

        let written = match self.project.engine {
            EngineKind::RpgMakerMv | EngineKind::RpgMakerMz => {
                game_translator_engine_rpgmaker::write_translations(
                    &self.project,
                    translations,
                    output_root,
                )
            }
            EngineKind::RenPy => game_translator_engine_renpy::write_translations(
                &self.project,
                translations,
                output_root,
                language,
            ),
        }
        .map_err(|error| PatchError::Engine(error.to_string()))?;
        let mut files = Vec::with_capacity(written.len());
        for target_path in written {
            let relative_path = target_path
                .strip_prefix(output_root)
                .map_err(|error| PatchError::Io {
                    path: target_path.clone(),
                    message: error.to_string(),
                })?
                .to_path_buf();
            let source_sha256 = if self.project.engine == EngineKind::RenPy {
                self.source_hashes
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or_default()
            } else {
                hash_file(&self.project.root.join(&relative_path))?
            };
            files.push(PatchFile {
                relative_path,
                source_sha256,
                target_sha256: hash_file(&target_path)?,
            });
        }
        let manifest = PatchManifest {
            format_version: 1,
            files,
        };
        fs::create_dir_all(output_root).map_err(|error| PatchError::Io {
            path: output_root.to_path_buf(),
            message: error.to_string(),
        })?;
        let manifest_path = output_root.join("patch-manifest.json");
        let rendered = serde_json::to_vec_pretty(&manifest).map_err(|error| PatchError::Io {
            path: manifest_path.clone(),
            message: error.to_string(),
        })?;
        fs::write(&manifest_path, rendered).map_err(|error| PatchError::Io {
            path: manifest_path,
            message: error.to_string(),
        })?;
        Ok(manifest)
    }
}

fn hash_file(path: &Path) -> Result<String, PatchError> {
    let bytes = fs::read(path).map_err(|error| PatchError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    Ok(encoded)
}
