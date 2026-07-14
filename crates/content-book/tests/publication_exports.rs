use std::{fs, io::Read};

use game_translator_content_book::{
    export_docx, export_epub, export_markdown, export_pdf, parse_book, print_dimensions_mm,
    BookExportProfile, PrintPreset,
};
use zip::ZipArchive;

fn translated_book(directory: &tempfile::TempDir) -> game_translator_content_book::BookProject {
    let source = directory.path().join("novel.md");
    fs::write(&source, "# 第一章\n\nSource paragraph.").unwrap();
    let mut project = parse_book(&source).unwrap();
    "这是译文。".clone_into(&mut project.chapters[0].segments[0].translation);
    "原作者".clone_into(&mut project.publication.author);
    "译者".clone_into(&mut project.publication.translator);
    "示例出版社".clone_into(&mut project.publication.publisher);
    "978-7-0000-0000-1".clone_into(&mut project.publication.isbn);
    project
}

#[test]
fn markdown_contains_publication_metadata_and_complete_manuscript() {
    let directory = tempfile::tempdir().unwrap();
    let project = translated_book(&directory);
    let output = directory.path().join("novel.md");

    export_markdown(&project, &output).unwrap();

    let markdown = fs::read_to_string(output).unwrap();
    assert!(markdown.contains("作者：原作者"));
    assert!(markdown.contains("译者：译者"));
    assert!(markdown.contains("ISBN：978-7-0000-0000-1"));
    assert!(markdown.contains("这是译文。"));
}

#[test]
fn docx_contains_word_styles_title_metadata_and_chapters() {
    let directory = tempfile::tempdir().unwrap();
    let project = translated_book(&directory);
    let output = directory.path().join("novel.docx");

    export_docx(&project, &output).unwrap();

    let mut archive = ZipArchive::new(fs::File::open(output).unwrap()).unwrap();
    assert!(archive.by_name("[Content_Types].xml").is_ok());
    let document = read_zip_text(&mut archive, "word/document.xml");
    let styles = read_zip_text(&mut archive, "word/styles.xml");
    assert!(document.contains("novel"));
    assert!(document.contains("原作者"));
    assert!(document.contains("第一章"));
    assert!(document.contains("这是译文。"));
    assert!(document.contains("w:pageBreakBefore"));
    assert!(styles.contains("w:styleId=\"BookBody\""));
    assert!(styles.contains("w:firstLineChars=\"200\""));
}

#[test]
fn epub3_contains_metadata_navigation_spine_and_chapter_xhtml() {
    let directory = tempfile::tempdir().unwrap();
    let project = translated_book(&directory);
    let output = directory.path().join("novel.epub");

    export_epub(&project, &output).unwrap();

    let mut archive = ZipArchive::new(fs::File::open(output).unwrap()).unwrap();
    assert_eq!(
        read_zip_text(&mut archive, "mimetype"),
        "application/epub+zip"
    );
    let package = read_zip_text(&mut archive, "OEBPS/content.opf");
    let navigation = read_zip_text(&mut archive, "OEBPS/nav.xhtml");
    let chapter = read_zip_text(&mut archive, "OEBPS/chapter-1.xhtml");
    assert!(package.contains("version=\"3.0\""));
    assert!(package.contains("原作者"));
    assert!(package.contains("译者"));
    assert!(package.contains("978-7-0000-0000-1"));
    assert!(navigation.contains("第一章"));
    assert!(chapter.contains("这是译文。"));
}

#[test]
fn print_presets_use_publication_page_dimensions() {
    assert_eq!(print_dimensions_mm(PrintPreset::Large32), (140.0, 203.0));
    assert_eq!(print_dimensions_mm(PrintPreset::A5), (148.0, 210.0));
    assert_eq!(print_dimensions_mm(PrintPreset::Sixteen), (185.0, 260.0));
}

#[test]
fn pdf_contains_embedded_chinese_text_and_multiple_publication_pages() {
    let directory = tempfile::tempdir().unwrap();
    let mut project = translated_book(&directory);
    project.chapters.push(project.chapters[0].clone());
    project.chapters[1].title = "第二章".to_owned();
    let output = directory.path().join("novel.pdf");
    let font = fs::read(r"C:\Windows\Fonts\NotoSerifSC-VF.ttf")
        .or_else(|_| fs::read(r"C:\Windows\Fonts\simfang.ttf"))
        .unwrap();

    export_pdf(&project, &output, &BookExportProfile::default(), &font).unwrap();

    let bytes = fs::read(output).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.windows(5).any(|value| value == b"/Font"));
    assert!(bytes.len() > 10_000);
}

fn read_zip_text(archive: &mut ZipArchive<fs::File>, name: &str) -> String {
    let mut value = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut value)
        .unwrap();
    value
}
