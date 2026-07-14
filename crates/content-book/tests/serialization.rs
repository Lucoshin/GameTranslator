use std::fs;

use game_translator_content_book::parse_book;

#[test]
fn json_contract_uses_frontend_field_names_and_status_values() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("contract.txt");
    fs::write(&source, "第一章 契约\n\nHello.").unwrap();
    let project = parse_book(&source).unwrap();

    let json = serde_json::to_value(project).unwrap();

    assert!(json.get("sourcePath").is_some());
    assert!(json.get("sourceLanguage").is_some());
    assert_eq!(json["publication"]["printPreset"], "large32");
    assert_eq!(json["publication"]["author"], "");
    assert_eq!(json["format"], "txt");
    assert_eq!(json["chapters"][0]["segments"][0]["status"], "untranslated");
    assert!(json["chapters"][0]["segments"][0].get("qaNote").is_some());
}

#[test]
fn older_projects_without_publication_metadata_still_load() {
    let json = r#"{
      "id":"old-book","sourcePath":"old.txt","title":"Old","format":"txt",
      "sourceLanguage":"auto","targetLanguage":"zh-CN","chapters":[]
    }"#;

    let project: game_translator_content_book::BookProject = serde_json::from_str(json).unwrap();

    assert_eq!(
        project.publication.print_preset,
        game_translator_content_book::PrintPreset::Large32
    );
    assert!(project.publication.author.is_empty());
}
