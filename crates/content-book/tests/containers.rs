use std::{fs::File, io::Write, path::Path};

use game_translator_content_book::{parse_book, BookFormat};
use zip::{write::SimpleFileOptions, ZipWriter};

#[test]
fn parses_epub_spine_in_reading_order() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.epub");
    write_zip(
        &path,
        &[
            (
                "META-INF/container.xml",
                r#"<?xml version="1.0"?><container><rootfiles><rootfile full-path="OEBPS/content.opf"/></rootfiles></container>"#,
            ),
            (
                "OEBPS/content.opf",
                r#"<?xml version="1.0"?><package><manifest><item id="c1" href="ch1.xhtml" media-type="application/xhtml+xml"/><item id="c2" href="ch2.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="c1"/><itemref idref="c2"/></spine></package>"#,
            ),
            (
                "OEBPS/ch1.xhtml",
                r"<html><body><h1>Chapter One</h1><p>First paragraph.</p><p>Second paragraph.</p></body></html>",
            ),
            (
                "OEBPS/ch2.xhtml",
                r"<html><body><h1>Chapter Two</h1><p>Last paragraph.</p></body></html>",
            ),
        ],
    );

    let project = parse_book(&path).unwrap();

    assert_eq!(project.format, BookFormat::Epub);
    assert_eq!(project.chapters.len(), 2);
    assert_eq!(project.chapters[0].title, "Chapter One");
    assert_eq!(project.chapters[0].segments[1].source, "Second paragraph.");
    assert_eq!(project.chapters[1].segments[0].source, "Last paragraph.");
}

#[test]
fn parses_docx_heading_styles_into_chapters() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.docx");
    write_zip(
        &path,
        &[(
            "word/document.xml",
            r#"<?xml version="1.0"?><w:document xmlns:w="urn:w"><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>第一章</w:t></w:r></w:p><w:p><w:r><w:t>第一段正文。</w:t></w:r></w:p><w:p><w:r><w:t>第二段</w:t></w:r><w:r><w:t>正文。</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>第二章</w:t></w:r></w:p><w:p><w:r><w:t>结尾。</w:t></w:r></w:p></w:body></w:document>"#,
        )],
    );

    let project = parse_book(&path).unwrap();

    assert_eq!(project.format, BookFormat::Docx);
    assert_eq!(project.chapters.len(), 2);
    assert_eq!(project.chapters[0].title, "第一章");
    assert_eq!(project.chapters[0].segments[1].source, "第二段正文。");
    assert_eq!(project.chapters[1].segments[0].source, "结尾。");
}

fn write_zip(path: &Path, entries: &[(&str, &str)]) {
    let file = File::create(path).unwrap();
    let mut archive = ZipWriter::new(file);
    for (name, content) in entries {
        archive
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(content.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
}
