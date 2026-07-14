use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fmt::Write as FmtWrite,
    fs::{self, File},
    io::{Read, Write as IoWrite},
    path::Path,
};

use encoding_rs::GBK;
use printpdf::{
    Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Pt, TextItem,
    TextMatrix,
};
use quick_xml::{events::Event, reader::Reader};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BookFormat {
    Txt,
    Markdown,
    Epub,
    Docx,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentStatus {
    Untranslated,
    Draft,
    Reviewed,
    Issue,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BookExportFormat {
    Markdown,
    Docx,
    Epub,
    Pdf,
}

impl BookExportFormat {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Docx => "docx",
            Self::Epub => "epub",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PrintPreset {
    #[default]
    Large32,
    A5,
    Sixteen,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PublicationMetadata {
    pub author: String,
    pub translator: String,
    pub publisher: String,
    pub isbn: String,
    pub copyright: String,
    pub cover_path: String,
    pub print_preset: PrintPreset,
}

impl Default for PublicationMetadata {
    fn default() -> Self {
        Self {
            author: String::new(),
            translator: String::new(),
            publisher: String::new(),
            isbn: String::new(),
            copyright: String::new(),
            cover_path: String::new(),
            print_preset: PrintPreset::Large32,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookExportProfile {
    pub print_preset: PrintPreset,
    pub include_page_numbers: bool,
    pub chapter_starts_new_page: bool,
}

impl Default for BookExportProfile {
    fn default() -> Self {
        Self {
            print_preset: PrintPreset::Large32,
            include_page_numbers: true,
            chapter_starts_new_page: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookExportRecord {
    pub id: String,
    pub project_id: String,
    pub book_title: String,
    pub format: BookExportFormat,
    pub output_path: String,
    pub target_language: String,
    pub exported_at_unix_ms: u64,
    pub profile: BookExportProfile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSegment {
    pub id: String,
    pub source: String,
    pub translation: String,
    pub status: SegmentStatus,
    pub qa_note: Option<String>,
    pub terms: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookChapter {
    pub id: String,
    pub title: String,
    pub segments: Vec<BookSegment>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookProject {
    pub id: String,
    pub source_path: String,
    pub title: String,
    pub format: BookFormat,
    pub source_language: String,
    pub target_language: String,
    pub chapters: Vec<BookChapter>,
    #[serde(default)]
    pub publication: PublicationMetadata,
}

#[derive(Debug)]
pub enum BookError {
    UnsupportedFormat(String),
    InvalidData(String),
    Io(String),
}

impl fmt::Display for BookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(value) => write!(formatter, "不支持的书籍格式：{value}"),
            Self::InvalidData(value) | Self::Io(value) => formatter.write_str(value),
        }
    }
}

impl Error for BookError {}

/// Parses a supported book without modifying the source file.
///
/// # Errors
/// Returns a readable error when the file is unavailable, unsupported, or has no body text.
pub fn parse_book(path: &Path) -> Result<BookProject, BookError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let format = match extension.as_str() {
        "txt" => BookFormat::Txt,
        "md" | "markdown" => BookFormat::Markdown,
        "epub" => BookFormat::Epub,
        "docx" => BookFormat::Docx,
        _ => return Err(BookError::UnsupportedFormat(extension)),
    };
    match format {
        BookFormat::Txt | BookFormat::Markdown => parse_plain_text(path, format),
        BookFormat::Epub => parse_epub(path),
        BookFormat::Docx => parse_docx(path),
    }
}

/// Exports the current book state as an independent UTF-8 Markdown document.
/// Untranslated segments retain their source text so the exported manuscript is complete.
///
/// # Errors
/// Returns an I/O error when the target document cannot be written.
pub fn export_markdown(project: &BookProject, path: &Path) -> Result<(), BookError> {
    let mut output = format!("# {}\n\n", project.title.trim());
    push_markdown_metadata(&mut output, "作者", &project.publication.author);
    push_markdown_metadata(&mut output, "译者", &project.publication.translator);
    push_markdown_metadata(&mut output, "出版社", &project.publication.publisher);
    push_markdown_metadata(&mut output, "ISBN", &project.publication.isbn);
    push_markdown_metadata(&mut output, "版权", &project.publication.copyright);
    if !project.publication.author.trim().is_empty()
        || !project.publication.translator.trim().is_empty()
        || !project.publication.publisher.trim().is_empty()
        || !project.publication.isbn.trim().is_empty()
        || !project.publication.copyright.trim().is_empty()
    {
        output.push('\n');
    }
    for chapter in &project.chapters {
        output.push_str("## ");
        output.push_str(chapter.title.trim());
        output.push_str("\n\n");
        for segment in &chapter.segments {
            let text = if segment.translation.trim().is_empty() {
                segment.source.trim()
            } else {
                segment.translation.trim()
            };
            if !text.is_empty() {
                output.push_str(text);
                output.push_str("\n\n");
            }
        }
    }
    fs::write(path, output).map_err(|error| BookError::Io(format!("导出 Markdown 失败：{error}")))
}

/// Exports an editable Office Open XML manuscript with publication styles.
///
/// # Errors
/// Returns an I/O error when the DOCX package cannot be written.
pub fn export_docx(project: &BookProject, path: &Path) -> Result<(), BookError> {
    let mut archive = ZipWriter::new(
        File::create(path).map_err(|error| BookError::Io(format!("创建 DOCX 失败：{error}")))?,
    );
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_zip_entry(
        &mut archive,
        "[Content_Types].xml",
        DOCX_CONTENT_TYPES,
        options,
    )?;
    write_zip_entry(&mut archive, "_rels/.rels", DOCX_ROOT_RELS, options)?;
    write_zip_entry(&mut archive, "word/styles.xml", DOCX_STYLES, options)?;
    write_zip_entry(
        &mut archive,
        "word/_rels/document.xml.rels",
        DOCX_DOCUMENT_RELS,
        options,
    )?;
    write_zip_entry(
        &mut archive,
        "word/document.xml",
        &docx_document(project),
        options,
    )?;
    archive
        .finish()
        .map_err(|error| BookError::Io(format!("完成 DOCX 失败：{error}")))?;
    Ok(())
}

/// Exports a standards-oriented EPUB 3 package with navigation and metadata.
///
/// # Errors
/// Returns an I/O error when the EPUB package cannot be written.
pub fn export_epub(project: &BookProject, path: &Path) -> Result<(), BookError> {
    let mut archive = ZipWriter::new(
        File::create(path).map_err(|error| BookError::Io(format!("创建 EPUB 失败：{error}")))?,
    );
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let compressed = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_zip_entry(&mut archive, "mimetype", "application/epub+zip", stored)?;
    write_zip_entry(
        &mut archive,
        "META-INF/container.xml",
        EPUB_CONTAINER,
        compressed,
    )?;
    write_zip_entry(&mut archive, "OEBPS/styles.css", EPUB_STYLES, compressed)?;
    write_zip_entry(
        &mut archive,
        "OEBPS/nav.xhtml",
        &epub_navigation(project),
        compressed,
    )?;
    write_zip_entry(
        &mut archive,
        "OEBPS/content.opf",
        &epub_package(project),
        compressed,
    )?;
    for (index, chapter) in project.chapters.iter().enumerate() {
        write_zip_entry(
            &mut archive,
            &format!("OEBPS/chapter-{}.xhtml", index + 1),
            &epub_chapter(project, chapter),
            compressed,
        )?;
    }
    archive
        .finish()
        .map_err(|error| BookError::Io(format!("完成 EPUB 失败：{error}")))?;
    Ok(())
}

/// Returns the finished-page dimensions for the supported Chinese publication presets.
#[must_use]
pub const fn print_dimensions_mm(preset: PrintPreset) -> (f32, f32) {
    match preset {
        PrintPreset::Large32 => (140.0, 203.0),
        PrintPreset::A5 => (148.0, 210.0),
        PrintPreset::Sixteen => (185.0, 260.0),
    }
}

/// Exports a paginated PDF manuscript using an embedded Chinese font.
///
/// # Errors
/// Returns a readable error when the supplied font cannot be parsed or the PDF cannot be written.
pub fn export_pdf(
    project: &BookProject,
    path: &Path,
    profile: &BookExportProfile,
    font_bytes: &[u8],
) -> Result<(), BookError> {
    let mut font_warnings = Vec::new();
    let font = ParsedFont::from_bytes(font_bytes, 0, &mut font_warnings)
        .ok_or_else(|| BookError::InvalidData("无法读取用于出版 PDF 的中文字体".to_owned()))?;
    let mut document = PdfDocument::new(&project.title);
    let font_id = document.add_font(&font);
    let (width, height) = print_dimensions_mm(profile.print_preset);
    let mut pages = Vec::new();

    let mut title_page = PdfComposer::new(width, height, font_id.clone(), profile);
    title_page.write_centered(&project.title, 20.0, 0.34);
    title_page.move_down(16.0);
    for line in publication_lines(project) {
        title_page.write_centered(&line, 11.0, 0.0);
        title_page.move_down(3.0);
    }
    pages.push(title_page.finish(None));

    let mut continuing_composer = None;
    let mut body_page_number = 0;
    for chapter in &project.chapters {
        let mut starts_fresh_page =
            profile.chapter_starts_new_page || continuing_composer.is_none();
        let mut composer = if profile.chapter_starts_new_page {
            PdfComposer::new(width, height, font_id.clone(), profile)
        } else {
            continuing_composer
                .take()
                .unwrap_or_else(|| PdfComposer::new(width, height, font_id.clone(), profile))
        };
        if !profile.chapter_starts_new_page && composer.y < height - 22.0 {
            composer.move_down(8.0);
        }
        if !composer.has_room() {
            body_page_number += 1;
            pages.push(composer.finish(Some(body_page_number)));
            composer = PdfComposer::new(width, height, font_id.clone(), profile);
            starts_fresh_page = true;
        }
        composer.write_centered(
            &chapter.title,
            16.0,
            if starts_fresh_page { 0.05 } else { 0.0 },
        );
        composer.move_down(10.0);
        for segment in &chapter.segments {
            let text = manuscript_text(segment);
            if text.is_empty() {
                continue;
            }
            for line in wrap_cjk(text, composer.characters_per_line()) {
                if !composer.has_room() {
                    body_page_number += 1;
                    pages.push(composer.finish(Some(body_page_number)));
                    composer = PdfComposer::new(width, height, font_id.clone(), profile);
                }
                composer.write_body_line(&line);
            }
            composer.move_down(2.0);
        }
        if profile.chapter_starts_new_page {
            body_page_number += 1;
            pages.push(composer.finish(Some(body_page_number)));
        } else {
            continuing_composer = Some(composer);
        }
    }
    if let Some(composer) = continuing_composer {
        body_page_number += 1;
        pages.push(composer.finish(Some(body_page_number)));
    }

    let bytes = document.with_pages(pages).save(
        &PdfSaveOptions {
            subset_fonts: true,
            ..PdfSaveOptions::default()
        },
        &mut Vec::new(),
    );
    fs::write(path, bytes).map_err(|error| BookError::Io(format!("导出 PDF 失败：{error}")))
}

fn publication_lines(project: &BookProject) -> Vec<String> {
    [
        ("作者", project.publication.author.as_str()),
        ("译者", project.publication.translator.as_str()),
        ("出版社", project.publication.publisher.as_str()),
        ("ISBN", project.publication.isbn.as_str()),
        ("版权", project.publication.copyright.as_str()),
    ]
    .into_iter()
    .filter(|(_, value)| !value.trim().is_empty())
    .map(|(label, value)| format!("{label}：{}", value.trim()))
    .collect()
}

struct PdfComposer<'a> {
    width: f32,
    height: f32,
    left: f32,
    right: f32,
    bottom: f32,
    y: f32,
    line_height: f32,
    font: printpdf::FontId,
    profile: &'a BookExportProfile,
    ops: Vec<Op>,
}

impl<'a> PdfComposer<'a> {
    fn new(
        width: f32,
        height: f32,
        font: printpdf::FontId,
        profile: &'a BookExportProfile,
    ) -> Self {
        Self {
            width,
            height,
            left: 22.0,
            right: 18.0,
            bottom: 18.0,
            y: height - 22.0,
            line_height: 6.4,
            font,
            profile,
            ops: vec![Op::StartTextSection],
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn characters_per_line(&self) -> usize {
        (((self.width - self.left - self.right) / 3.9).floor() as usize).max(12)
    }

    fn has_room(&self) -> bool {
        self.y - self.line_height > self.bottom
    }

    fn move_down(&mut self, mm: f32) {
        self.y -= mm;
    }

    #[allow(clippy::cast_precision_loss)]
    fn write_centered(&mut self, text: &str, size: f32, relative_y: f32) {
        if relative_y > 0.0 {
            self.y = self.height * (1.0 - relative_y);
        }
        let estimated_width = text.chars().count() as f32 * size * 0.36;
        let x = ((self.width - estimated_width) / 2.0).max(self.left);
        self.write_at(text, x, self.y, size);
        self.y -= size * 0.45;
    }

    fn write_body_line(&mut self, text: &str) {
        self.write_at(text, self.left, self.y, 10.5);
        self.y -= self.line_height;
    }

    fn write_at(&mut self, text: &str, x: f32, y: f32, size: f32) {
        self.ops.push(Op::SetTextMatrix {
            matrix: TextMatrix::Translate(Mm(x).into_pt(), Mm(y).into_pt()),
        });
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.font.clone()),
            size: Pt(size),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.to_owned())],
        });
    }

    #[allow(clippy::cast_precision_loss)]
    fn finish(mut self, page_number: Option<usize>) -> PdfPage {
        if self.profile.include_page_numbers && page_number.is_some() {
            let page_number = page_number.unwrap_or_default();
            let label = page_number.to_string();
            let x = self.width / 2.0 - label.len() as f32;
            self.write_at(&label, x, 9.0, 8.0);
        }
        self.ops.push(Op::EndTextSection);
        PdfPage::new(Mm(self.width), Mm(self.height), self.ops)
    }
}

fn wrap_cjk(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let chars = paragraph.trim().chars().collect::<Vec<_>>();
        if chars.is_empty() {
            continue;
        }
        let mut start = 0;
        let mut line_index = 0;
        while start < chars.len() {
            let remaining = chars.len() - start;
            let mut take = remaining.min(max_chars);
            let tail = remaining.saturating_sub(take);
            if tail > 0 && tail < 4 {
                take = take.saturating_sub(4 - tail).max(1);
            }
            while start + take < chars.len()
                && "，。！？；：、）》】」』…".contains(chars[start + take])
                && take < remaining
            {
                take += 1;
            }
            let mut line = chars[start..start + take].iter().collect::<String>();
            if line_index == 0 {
                line.insert_str(0, "　　");
            }
            lines.push(line);
            start += take;
            line_index += 1;
        }
    }
    lines
}

fn push_markdown_metadata(output: &mut String, label: &str, value: &str) {
    if !value.trim().is_empty() {
        let _ = writeln!(output, "{label}：{}  ", value.trim());
    }
}

fn manuscript_text(segment: &BookSegment) -> &str {
    if segment.translation.trim().is_empty() {
        segment.source.trim()
    } else {
        segment.translation.trim()
    }
}

fn write_zip_entry(
    archive: &mut ZipWriter<File>,
    name: &str,
    content: &str,
    options: SimpleFileOptions,
) -> Result<(), BookError> {
    archive
        .start_file(name, options)
        .map_err(|error| BookError::Io(format!("写入 {name} 失败：{error}")))?;
    archive
        .write_all(content.as_bytes())
        .map_err(|error| BookError::Io(format!("写入 {name} 失败：{error}")))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn docx_document(project: &BookProject) -> String {
    let mut body = String::new();
    body.push_str(&docx_paragraph(&project.title, "BookTitle", false));
    for value in [
        (!project.publication.author.is_empty())
            .then(|| format!("作者：{}", project.publication.author)),
        (!project.publication.translator.is_empty())
            .then(|| format!("译者：{}", project.publication.translator)),
        (!project.publication.publisher.is_empty())
            .then(|| format!("出版社：{}", project.publication.publisher)),
        (!project.publication.isbn.is_empty())
            .then(|| format!("ISBN：{}", project.publication.isbn)),
        (!project.publication.copyright.is_empty()).then(|| project.publication.copyright.clone()),
    ]
    .into_iter()
    .flatten()
    {
        body.push_str(&docx_paragraph(&value, "BookMeta", false));
    }
    for chapter in &project.chapters {
        body.push_str(&docx_paragraph(&chapter.title, "ChapterTitle", true));
        for segment in &chapter.segments {
            let text = manuscript_text(segment);
            if !text.is_empty() {
                body.push_str(&docx_paragraph(text, "BookBody", false));
            }
        }
    }
    let (width, height) = print_dimensions_mm(project.publication.print_preset);
    let width_twips = mm_to_twips(width);
    let height_twips = mm_to_twips(height);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="{width_twips}" w:h="{height_twips}"/><w:pgMar w:top="1134" w:right="1021" w:bottom="1134" w:left="1134" w:gutter="227"/></w:sectPr></w:body></w:document>"#
    )
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn mm_to_twips(value: f32) -> u32 {
    (value * 1440.0 / 25.4).round() as u32
}

fn docx_paragraph(text: &str, style: &str, page_break_before: bool) -> String {
    let page_break = if page_break_before {
        "<w:pageBreakBefore/>"
    } else {
        ""
    };
    format!(
        r#"<w:p><w:pPr><w:pStyle w:val="{style}"/>{page_break}</w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
        xml_escape(text)
    )
}

fn epub_navigation(project: &BookProject) -> String {
    let items =
        project
            .chapters
            .iter()
            .enumerate()
            .fold(String::new(), |mut output, (index, chapter)| {
                let _ = write!(
                    output,
                    r#"<li><a href="chapter-{}.xhtml">{}</a></li>"#,
                    index + 1,
                    xml_escape(&chapter.title)
                );
                output
            });
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="{}"><head><title>目录</title></head><body><nav epub:type="toc" id="toc"><h1>目录</h1><ol>{items}</ol></nav></body></html>"#,
        xml_escape(&project.target_language)
    )
}

fn epub_package(project: &BookProject) -> String {
    let identifier = if project.publication.isbn.trim().is_empty() {
        project.id.as_str()
    } else {
        project.publication.isbn.trim()
    };
    let manifest = project
        .chapters
        .iter()
        .enumerate()
        .fold(String::new(), |mut output, (index, _)| {
            let _ = write!(output, r#"<item id="chapter-{0}" href="chapter-{0}.xhtml" media-type="application/xhtml+xml"/>"#, index + 1);
            output
        });
    let spine =
        project
            .chapters
            .iter()
            .enumerate()
            .fold(String::new(), |mut output, (index, _)| {
                let _ = write!(output, r#"<itemref idref="chapter-{}"/>"#, index + 1);
                output
            });
    let author = if project.publication.author.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<dc:creator>{}</dc:creator>",
            xml_escape(&project.publication.author)
        )
    };
    let translator = if project.publication.translator.trim().is_empty() {
        String::new()
    } else {
        format!(
            r##"<dc:contributor id="translator">{}</dc:contributor><meta refines="#translator" property="role" scheme="marc:relators">trl</meta>"##,
            xml_escape(&project.publication.translator)
        )
    };
    let publisher = if project.publication.publisher.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<dc:publisher>{}</dc:publisher>",
            xml_escape(&project.publication.publisher)
        )
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="book-id">{}</dc:identifier><dc:title>{}</dc:title><dc:language>{}</dc:language>{author}{translator}{publisher}<meta property="dcterms:modified">2026-07-14T00:00:00Z</meta></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="css" href="styles.css" media-type="text/css"/>{manifest}</manifest><spine>{spine}</spine></package>"#,
        xml_escape(identifier),
        xml_escape(&project.title),
        xml_escape(&project.target_language)
    )
}

fn epub_chapter(project: &BookProject, chapter: &BookChapter) -> String {
    let paragraphs = chapter
        .segments
        .iter()
        .filter_map(|segment| {
            let text = manuscript_text(segment);
            (!text.is_empty()).then(|| format!("<p>{}</p>", xml_escape(text)))
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" lang="{}"><head><title>{}</title><link rel="stylesheet" type="text/css" href="styles.css"/></head><body><section epub:type="chapter" xmlns:epub="http://www.idpf.org/2007/ops"><h1>{}</h1>{paragraphs}</section></body></html>"#,
        xml_escape(&project.target_language),
        xml_escape(&chapter.title),
        xml_escape(&chapter.title)
    )
}

const DOCX_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#;
const DOCX_ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
const DOCX_DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
const DOCX_STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Noto Serif SC" w:eastAsia="Noto Serif SC"/><w:sz w:val="21"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:styleId="BookTitle"><w:name w:val="书名"/><w:pPr><w:jc w:val="center"/><w:spacing w:before="2400" w:after="720"/></w:pPr><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="BookMeta"><w:name w:val="出版信息"/><w:pPr><w:jc w:val="center"/><w:spacing w:after="180"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="ChapterTitle"><w:name w:val="章节标题"/><w:pPr><w:jc w:val="center"/><w:spacing w:before="720" w:after="720"/></w:pPr><w:rPr><w:b/><w:sz w:val="30"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="BookBody"><w:name w:val="正文"/><w:pPr><w:ind w:firstLineChars="200"/><w:spacing w:line="360" w:lineRule="auto" w:after="0"/></w:pPr><w:rPr><w:sz w:val="21"/></w:rPr></w:style></w:styles>"#;
const EPUB_CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
const EPUB_STYLES: &str = "html{writing-mode:horizontal-tb;}body{font-family:serif;line-height:1.8;margin:5%;}h1{text-align:center;page-break-before:always;margin:2em 0;}p{text-indent:2em;margin:0;}";

fn parse_epub(path: &Path) -> Result<BookProject, BookError> {
    let file =
        File::open(path).map_err(|error| BookError::Io(format!("读取 EPUB 失败：{error}")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| BookError::InvalidData(format!("EPUB 容器无效：{error}")))?;
    let container = read_archive_text(&mut archive, "META-INF/container.xml")?;
    let rootfile = attribute_from_element(&container, "rootfile", "full-path")
        .ok_or_else(|| BookError::InvalidData("EPUB 缺少 OPF 根文件".to_owned()))?;
    let package = read_archive_text(&mut archive, &rootfile)?;
    let manifest = epub_manifest(&package);
    let spine = epub_spine(&package);
    let package_root = Path::new(&rootfile)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let mut chapters = Vec::new();
    for (index, item_id) in spine.iter().enumerate() {
        let Some(href) = manifest.get(item_id) else {
            continue;
        };
        let entry = archive_path(&package_root.join(href));
        let xhtml = read_archive_text(&mut archive, &entry)?;
        let blocks = xml_blocks(&xhtml, XmlMode::Html)?;
        let title = blocks
            .iter()
            .find(|block| block.heading)
            .map_or_else(|| format!("章节 {}", index + 1), |block| block.text.clone());
        let paragraphs = blocks
            .into_iter()
            .filter(|block| !block.heading)
            .map(|block| block.text)
            .collect::<Vec<_>>();
        if !paragraphs.is_empty() {
            chapters.push((title, paragraphs));
        }
    }
    build_project(path, BookFormat::Epub, chapters)
}

fn parse_docx(path: &Path) -> Result<BookProject, BookError> {
    let file =
        File::open(path).map_err(|error| BookError::Io(format!("读取 DOCX 失败：{error}")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| BookError::InvalidData(format!("DOCX 容器无效：{error}")))?;
    let document = read_archive_text(&mut archive, "word/document.xml")?;
    let blocks = xml_blocks(&document, XmlMode::Docx)?;
    let mut chapters = Vec::<(String, Vec<String>)>::new();
    for block in blocks {
        if block.heading {
            chapters.push((block.text, Vec::new()));
        } else {
            if chapters.is_empty() {
                chapters.push(("正文".to_owned(), Vec::new()));
            }
            if let Some((_, paragraphs)) = chapters.last_mut() {
                paragraphs.push(block.text);
            }
        }
    }
    build_project(path, BookFormat::Docx, chapters)
}

fn read_archive_text(archive: &mut ZipArchive<File>, name: &str) -> Result<String, BookError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|error| BookError::InvalidData(format!("书籍容器缺少 {name}：{error}")))?;
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|error| BookError::InvalidData(format!("读取 {name} 失败：{error}")))?;
    Ok(content)
}

fn attribute_from_element(xml: &str, element_name: &str, attribute_name: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event().ok()? {
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) == element_name =>
            {
                return element.attributes().flatten().find_map(|attribute| {
                    (local_name(attribute.key.as_ref()) == attribute_name)
                        .then(|| String::from_utf8_lossy(attribute.value.as_ref()).into_owned())
                });
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

fn epub_manifest(xml: &str) -> HashMap<String, String> {
    let mut reader = Reader::from_str(xml);
    let mut items = HashMap::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element))
                if local_name(element.name().as_ref()) == "item" =>
            {
                let mut id = None;
                let mut href = None;
                for attribute in element.attributes().flatten() {
                    let value = String::from_utf8_lossy(attribute.value.as_ref()).into_owned();
                    match local_name(attribute.key.as_ref()) {
                        "id" => id = Some(value),
                        "href" => href = Some(value),
                        _ => {}
                    }
                }
                if let (Some(id), Some(href)) = (id, href) {
                    items.insert(id, href);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    items
}

fn epub_spine(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    let mut items = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element))
                if local_name(element.name().as_ref()) == "itemref" =>
            {
                if let Some(value) = element.attributes().flatten().find_map(|attribute| {
                    (local_name(attribute.key.as_ref()) == "idref")
                        .then(|| String::from_utf8_lossy(attribute.value.as_ref()).into_owned())
                }) {
                    items.push(value);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    items
}

#[derive(Clone, Copy)]
enum XmlMode {
    Html,
    Docx,
}

struct XmlBlock {
    text: String,
    heading: bool,
}

fn xml_blocks(xml: &str, mode: XmlMode) -> Result<Vec<XmlBlock>, BookError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut blocks = Vec::new();
    let mut current = None::<XmlBlock>;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                let name = local_name(element.name().as_ref()).to_owned();
                let starts_block = match mode {
                    XmlMode::Html => name == "p" || is_html_heading(&name),
                    XmlMode::Docx => name == "p",
                };
                if starts_block {
                    current = Some(XmlBlock {
                        text: String::new(),
                        heading: matches!(mode, XmlMode::Html) && is_html_heading(&name),
                    });
                } else if matches!(mode, XmlMode::Docx) && name == "pStyle" {
                    let heading = element.attributes().flatten().any(|attribute| {
                        local_name(attribute.key.as_ref()) == "val"
                            && String::from_utf8_lossy(attribute.value.as_ref())
                                .to_ascii_lowercase()
                                .starts_with("heading")
                    });
                    if heading {
                        if let Some(block) = current.as_mut() {
                            block.heading = true;
                        }
                    }
                }
            }
            Ok(Event::Empty(element)) if matches!(mode, XmlMode::Docx) => {
                if local_name(element.name().as_ref()) == "pStyle" {
                    let heading = element.attributes().flatten().any(|attribute| {
                        local_name(attribute.key.as_ref()) == "val"
                            && String::from_utf8_lossy(attribute.value.as_ref())
                                .to_ascii_lowercase()
                                .starts_with("heading")
                    });
                    if heading {
                        if let Some(block) = current.as_mut() {
                            block.heading = true;
                        }
                    }
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(block) = current.as_mut() {
                    let value = text.decode().map_err(|error| {
                        BookError::InvalidData(format!("XML 文本无效：{error}"))
                    })?;
                    block.text.push_str(&value);
                }
            }
            Ok(Event::End(element)) => {
                let name = local_name(element.name().as_ref()).to_owned();
                let ends_block = match mode {
                    XmlMode::Html => name == "p" || is_html_heading(&name),
                    XmlMode::Docx => name == "p",
                };
                if ends_block {
                    if let Some(mut block) = current.take() {
                        let text = std::mem::take(&mut block.text);
                        text.trim().clone_into(&mut block.text);
                        if !block.text.is_empty() {
                            blocks.push(block);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(BookError::InvalidData(format!("书籍 XML 无效：{error}"))),
            _ => {}
        }
    }
    Ok(blocks)
}

fn local_name(value: &[u8]) -> &str {
    let value = std::str::from_utf8(value).unwrap_or_default();
    value.rsplit(':').next().unwrap_or(value)
}

fn is_html_heading(name: &str) -> bool {
    matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

fn archive_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn parse_plain_text(path: &Path, format: BookFormat) -> Result<BookProject, BookError> {
    let bytes = fs::read(path).map_err(|error| BookError::Io(format!("读取书籍失败：{error}")))?;
    let text = decode_text(&bytes)?;
    let blocks = split_blocks(&text);
    let mut chapters = Vec::<(String, Vec<String>)>::new();
    for block in blocks {
        let heading = match format {
            BookFormat::Markdown => markdown_heading(&block),
            BookFormat::Txt => txt_heading(&block),
            BookFormat::Epub | BookFormat::Docx => None,
        };
        if let Some(title) = heading {
            chapters.push((title, Vec::new()));
        } else {
            if chapters.is_empty() {
                chapters.push(("正文".to_owned(), Vec::new()));
            }
            if let Some((_, paragraphs)) = chapters.last_mut() {
                paragraphs.push(block);
            }
        }
    }
    build_project(path, format, chapters)
}

fn build_project(
    path: &Path,
    format: BookFormat,
    chapters: Vec<(String, Vec<String>)>,
) -> Result<BookProject, BookError> {
    let chapters = chapters
        .into_iter()
        .filter(|(_, paragraphs)| !paragraphs.is_empty())
        .enumerate()
        .map(|(chapter_index, (title, paragraphs))| {
            let chapter_id = stable_id(&format!(
                "{}\0chapter\0{chapter_index}\0{title}",
                path.display()
            ));
            let segments = paragraphs
                .into_iter()
                .enumerate()
                .map(|(segment_index, source)| BookSegment {
                    id: stable_id(&format!("{chapter_id}\0{segment_index}\0{source}")),
                    source,
                    translation: String::new(),
                    status: SegmentStatus::Untranslated,
                    qa_note: None,
                    terms: Vec::new(),
                })
                .collect();
            BookChapter {
                id: chapter_id,
                title,
                segments,
            }
        })
        .collect::<Vec<_>>();
    if chapters.is_empty() {
        return Err(BookError::InvalidData("书籍没有可导入的正文".to_owned()));
    }
    let source_path = path.to_string_lossy().into_owned();
    let title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("未命名书稿")
        .to_owned();
    Ok(BookProject {
        id: stable_id(&source_path),
        source_path,
        title,
        format,
        source_language: "auto".to_owned(),
        target_language: "zh-CN".to_owned(),
        chapters,
        publication: PublicationMetadata::default(),
    })
}

fn decode_text(bytes: &[u8]) -> Result<String, BookError> {
    if let Some(content) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(content.to_vec())
            .map_err(|error| BookError::InvalidData(format!("UTF-8 文本无效：{error}")));
    }
    if let Some(content) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let units = content
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .map_err(|error| BookError::InvalidData(format!("UTF-16 文本无效：{error}")));
    }
    if let Ok(value) = std::str::from_utf8(bytes) {
        return Ok(value.to_owned());
    }
    let (decoded, _, had_errors) = GBK.decode(bytes);
    if had_errors {
        Err(BookError::InvalidData("无法识别书籍文本编码".to_owned()))
    } else {
        Ok(decoded.into_owned())
    }
}

fn split_blocks(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split("\n\n")
        .map(|block| block.lines().map(str::trim).collect::<Vec<_>>().join("\n"))
        .filter(|block| !block.is_empty())
        .collect()
}

fn markdown_heading(block: &str) -> Option<String> {
    let line = block.lines().next()?.trim();
    let marker_count = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    (marker_count > 0 && marker_count <= 6)
        .then(|| {
            line[marker_count..]
                .trim()
                .trim_end_matches('#')
                .trim()
                .to_owned()
        })
        .filter(|title| !title.is_empty())
}

fn txt_heading(block: &str) -> Option<String> {
    if block.contains('\n') || block.chars().count() > 80 {
        return None;
    }
    let lower = block.to_ascii_lowercase();
    let chinese = block.starts_with('第')
        && ["章", "节", "卷", "部", "篇"]
            .iter()
            .any(|marker| block.contains(marker));
    let latin = ["chapter ", "part ", "book "]
        .iter()
        .any(|prefix| lower.starts_with(prefix));
    (chinese || latin).then(|| block.trim().to_owned())
}

fn stable_id(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{digest:x}")[..24].to_owned()
}
