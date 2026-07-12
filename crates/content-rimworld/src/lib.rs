use std::{collections::BTreeMap, fs, hash::BuildHasher, path::Path, path::PathBuf};

use game_translator_content_core::{
    ContentCategory, ContentError, ContentOutputAdapter, ContentSource, ContentSourceAdapter,
    ExportRequest, ExportResult, OutputCapability, Segment, SegmentContext, SegmentKind,
};
use quick_xml::{events::Event, reader::Reader};

const FORMAT_ID: &str = "game.rimworld.mod";
const ENGLISH_LANGUAGE_DIRECTORY: &str = "Languages/English";

pub struct RimWorldModContentAdapter;
pub struct RimWorldLanguagePackOutputAdapter;

impl ContentSourceAdapter for RimWorldModContentAdapter {
    fn format_id(&self) -> &'static str {
        FORMAT_ID
    }

    fn category(&self) -> ContentCategory {
        ContentCategory::GameMod
    }

    fn output_capabilities(&self) -> &'static [OutputCapability] {
        &[OutputCapability::Export]
    }

    fn detect(&self, root: &Path) -> Result<ContentSource, ContentError> {
        let metadata_path = root.join("About/About.xml");
        if !metadata_path.is_file() {
            return Err(ContentError::UnsupportedSource);
        }

        let metadata = read_language_entries(&metadata_path, "ModMetaData")?;
        let package_id = metadata
            .iter()
            .find(|entry| entry.key.eq_ignore_ascii_case("packageId"))
            .map(|entry| entry.value.clone())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ContentError::InvalidData {
                path: metadata_path.clone(),
                message: "ModMetaData must contain a non-empty packageId".to_owned(),
            })?;
        let display_name = metadata
            .iter()
            .find(|entry| entry.key == "name")
            .map(|entry| entry.value.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| package_id.clone());

        Ok(ContentSource {
            root: root.to_path_buf(),
            format_id: self.format_id(),
            display_name,
            source_id: package_id,
        })
    }

    fn extract(&self, source: &ContentSource) -> Result<Vec<Segment>, ContentError> {
        let language_root = source.root.join(ENGLISH_LANGUAGE_DIRECTORY);
        if !language_root.is_dir() {
            return Err(ContentError::MissingRequiredFile(language_root));
        }

        let mut sources = Vec::new();
        for (category, directory) in [("keyed", "Keyed"), ("definjected", "DefInjected")] {
            let directory_path = language_root.join(directory);
            if directory_path.is_dir() {
                collect_xml_files(&directory_path, category, &mut sources)?;
            }
        }

        if sources.is_empty() {
            return Err(ContentError::MissingRequiredFile(language_root));
        }

        sources.sort_by(|left, right| left.path.cmp(&right.path));
        let mut segments = Vec::new();
        for source_file in sources {
            let relative_path = source_file
                .path
                .strip_prefix(&language_root)
                .map_err(|error| ContentError::InvalidData {
                    path: source_file.path.clone(),
                    message: error.to_string(),
                })?;
            for entry in read_language_entries(&source_file.path, "LanguageData")? {
                segments.push(Segment {
                    id: format!(
                        "rimworld:{}:{}:{}",
                        source_file.category,
                        stable_relative_path(relative_path),
                        entry.key
                    ),
                    source: entry.value,
                    source_file: source_file.path.clone(),
                    location: entry.key,
                    kind: SegmentKind::LocalizedKey,
                    context: SegmentContext {
                        speaker: None,
                        previous_text: None,
                        next_text: None,
                    },
                });
            }
        }
        Ok(segments)
    }
}

impl ContentOutputAdapter for RimWorldLanguagePackOutputAdapter {
    fn format_id(&self) -> &'static str {
        FORMAT_ID
    }

    fn capabilities(&self) -> &'static [OutputCapability] {
        &[OutputCapability::Export]
    }

    fn export<S: BuildHasher>(
        &self,
        request: &ExportRequest<'_, S>,
    ) -> Result<ExportResult, ContentError> {
        if request.source.format_id != FORMAT_ID {
            return Err(ContentError::UnsupportedSource);
        }
        let language_directory = language_directory(request.target_language, &request.source.root)?;
        let language_root = request
            .output_root
            .join("Languages")
            .join(language_directory);
        let source_language_root = request.source.root.join(ENGLISH_LANGUAGE_DIRECTORY);
        let segments = RimWorldModContentAdapter.extract(request.source)?;
        let mut files = BTreeMap::<PathBuf, Vec<(&Segment, &String)>>::new();
        for segment in &segments {
            if let Some(translation) = request.translations.get(&segment.id) {
                let relative = segment
                    .source_file
                    .strip_prefix(&source_language_root)
                    .map_err(|error| ContentError::InvalidData {
                        path: segment.source_file.clone(),
                        message: error.to_string(),
                    })?;
                files
                    .entry(relative.to_path_buf())
                    .or_default()
                    .push((segment, translation));
            }
        }

        let mut written = Vec::new();
        for (relative, entries) in files {
            let path = language_root.join(relative);
            write_language_file(&path, &entries)?;
            written.push(path);
        }
        let info_path = language_root.join("LanguageInfo.xml");
        write_language_info(&info_path, language_directory)?;
        written.push(info_path);
        Ok(ExportResult {
            output_root: request.output_root.to_path_buf(),
            files: written,
        })
    }
}

struct SourceFile {
    category: &'static str,
    path: PathBuf,
}

struct LanguageEntry {
    key: String,
    value: String,
}

fn collect_xml_files(
    directory: &Path,
    category: &'static str,
    files: &mut Vec<SourceFile>,
) -> Result<(), ContentError> {
    for entry in fs::read_dir(directory).map_err(|error| ContentError::Io {
        path: directory.to_path_buf(),
        message: error.to_string(),
    })? {
        let entry = entry.map_err(|error| ContentError::Io {
            path: directory.to_path_buf(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_xml_files(&path, category, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        {
            files.push(SourceFile { category, path });
        }
    }
    Ok(())
}

fn read_language_entries(
    path: &Path,
    expected_root: &str,
) -> Result<Vec<LanguageEntry>, ContentError> {
    let xml = fs::read_to_string(path).map_err(|error| ContentError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut root = None;
    let mut current = None;
    let mut entries = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                let name = decode_name(element.name().as_ref(), path)?;
                if root.is_none() {
                    if name != expected_root {
                        return Err(invalid_xml(
                            path,
                            format!("expected <{expected_root}> root element"),
                        ));
                    }
                    root = Some(name);
                } else if current.is_none() {
                    current = Some(LanguageEntry {
                        key: name,
                        value: String::new(),
                    });
                } else {
                    return Err(invalid_xml(
                        path,
                        "nested localization values are not supported".to_owned(),
                    ));
                }
            }
            Ok(Event::Empty(element)) => {
                if root.is_none() {
                    return Err(invalid_xml(
                        path,
                        format!("expected <{expected_root}> root element"),
                    ));
                }
                if current.is_some() {
                    return Err(invalid_xml(
                        path,
                        "nested localization values are not supported".to_owned(),
                    ));
                }
                let _ = decode_name(element.name().as_ref(), path)?;
            }
            Ok(Event::Text(text)) => {
                let value = text.decode().map_err(|error| ContentError::InvalidData {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                })?;
                if let Some(entry) = current.as_mut() {
                    entry.value.push_str(&value);
                } else if !value.trim().is_empty() {
                    return Err(invalid_xml(path, "unexpected text node".to_owned()));
                }
            }
            Ok(Event::End(element)) => {
                let name = decode_name(element.name().as_ref(), path)?;
                if let Some(entry) = current.take() {
                    if name != entry.key {
                        return Err(invalid_xml(
                            path,
                            "mismatched localization element".to_owned(),
                        ));
                    }
                    if !entry.value.trim().is_empty() {
                        entries.push(entry);
                    }
                } else if root.as_deref() == Some(name.as_str()) {
                    root = None;
                } else {
                    return Err(invalid_xml(path, "unexpected closing element".to_owned()));
                }
            }
            Ok(Event::Eof) => {
                if root.is_some() || current.is_some() {
                    return Err(invalid_xml(
                        path,
                        "unexpected end of XML document".to_owned(),
                    ));
                }
                return Ok(entries);
            }
            Ok(Event::CData(_)) => {
                return Err(unsupported_cdata(path));
            }
            Ok(_) => {}
            Err(error) => {
                return Err(ContentError::InvalidData {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                });
            }
        }
    }
}

fn stable_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn language_directory<'a>(language: &'a str, source_root: &Path) -> Result<&'a str, ContentError> {
    match language {
        "zh-CN" | "ChineseSimplified" => Ok("ChineseSimplified"),
        _ => Err(ContentError::InvalidData {
            path: source_root.to_path_buf(),
            message: "RimWorld language pack export currently supports only zh-CN".to_owned(),
        }),
    }
}

fn write_language_file(path: &Path, entries: &[(&Segment, &String)]) -> Result<(), ContentError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ContentError::Io {
            path: parent.to_path_buf(),
            message: error.to_string(),
        })?;
    }
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<LanguageData>\n");
    for (segment, translation) in entries {
        xml.push_str("  <");
        xml.push_str(&segment.location);
        xml.push('>');
        xml.push_str(&xml_escape(translation));
        xml.push_str("</");
        xml.push_str(&segment.location);
        xml.push_str(">\n");
    }
    xml.push_str("</LanguageData>\n");
    fs::write(path, xml).map_err(|error| ContentError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn write_language_info(path: &Path, language: &str) -> Result<(), ContentError> {
    fs::write(
        path,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<LanguageInfo>\n  <friendlyNameNative>{language}</friendlyNameNative>\n  <friendlyNameEnglish>{language}</friendlyNameEnglish>\n  <canBeTiny>true</canBeTiny>\n  <languageWorkerClass>LanguageWorker_English</languageWorkerClass>\n</LanguageInfo>\n"
        ),
    )
    .map_err(|error| ContentError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn decode_name(bytes: &[u8], path: &Path) -> Result<String, ContentError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| ContentError::InvalidData {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

fn invalid_xml(path: &Path, message: String) -> ContentError {
    ContentError::InvalidData {
        path: path.to_path_buf(),
        message,
    }
}

fn unsupported_cdata(path: &Path) -> ContentError {
    invalid_xml(
        path,
        "CDATA localization values are not supported".to_owned(),
    )
}
