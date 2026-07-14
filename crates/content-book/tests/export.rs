use std::fs;

use game_translator_content_book::{export_markdown, parse_book, SegmentStatus};

#[test]
fn exports_translations_and_falls_back_to_source_text() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("letters.md");
    let output = directory.path().join("letters.zh-CN.md");
    fs::write(
        &source,
        "# Chapter One\n\nThe harbor was quiet.\n\nA bell rang.",
    )
    .unwrap();
    let mut project = parse_book(&source).unwrap();
    project.chapters[0].segments[0].translation = "港口很安静。".to_owned();
    project.chapters[0].segments[0].status = SegmentStatus::Reviewed;

    export_markdown(&project, &output).unwrap();

    let markdown = fs::read_to_string(output).unwrap();
    assert!(markdown.starts_with("# letters\n\n## Chapter One"));
    assert!(markdown.contains("港口很安静。"));
    assert!(markdown.contains("A bell rang."));
}
