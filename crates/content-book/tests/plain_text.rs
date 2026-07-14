use std::fs;

use game_translator_content_book::{parse_book, BookFormat, SegmentStatus};

#[test]
fn parses_txt_chapter_titles_and_paragraphs() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("海边来信.txt");
    fs::write(
        &path,
        "第一章 雾港\n\nThe harbor was quiet.\n\nA bell rang.\n\n第二章 回信\n\nShe opened the letter.",
    )
    .unwrap();

    let project = parse_book(&path).unwrap();

    assert_eq!(project.title, "海边来信");
    assert_eq!(project.format, BookFormat::Txt);
    assert_eq!(project.chapters.len(), 2);
    assert_eq!(project.chapters[0].title, "第一章 雾港");
    assert_eq!(project.chapters[0].segments.len(), 2);
    assert_eq!(
        project.chapters[0].segments[0].source,
        "The harbor was quiet."
    );
    assert_eq!(
        project.chapters[0].segments[0].status,
        SegmentStatus::Untranslated
    );
}

#[test]
fn parses_markdown_headings_without_emitting_them_as_body_segments() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("notes.md");
    fs::write(
        &path,
        "# Part One\n\nFirst paragraph.\n\nSecond paragraph.\n\n## Part Two\n\nLast paragraph.",
    )
    .unwrap();

    let project = parse_book(&path).unwrap();

    assert_eq!(project.format, BookFormat::Markdown);
    assert_eq!(project.chapters.len(), 2);
    assert_eq!(project.chapters[0].title, "Part One");
    assert_eq!(project.chapters[1].segments[0].source, "Last paragraph.");
}

#[test]
fn rejects_an_empty_book() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("empty.txt");
    fs::write(&path, "  \n\n").unwrap();

    let error = parse_book(&path).unwrap_err();

    assert!(error.to_string().contains("没有可导入的正文"));
}
