use game_translator_project_store::{CacheEntry, ProjectStore};

#[test]
fn returns_a_translation_only_for_the_exact_input_fingerprint() {
    let store = ProjectStore::open_in_memory().unwrap();
    store
        .put_cache(&CacheEntry {
            input_fingerprint: "source+context+glossary+model-a".into(),
            translation: "月光石".into(),
        })
        .unwrap();

    assert_eq!(
        store
            .cached_translation("source+context+glossary+model-a")
            .unwrap()
            .as_deref(),
        Some("月光石")
    );
    assert_eq!(
        store
            .cached_translation("source+context+glossary+model-b")
            .unwrap(),
        None
    );
}

#[test]
fn replaces_a_cached_value_for_the_same_fingerprint() {
    let store = ProjectStore::open_in_memory().unwrap();
    for translation in ["月石", "月光石"] {
        store
            .put_cache(&CacheEntry {
                input_fingerprint: "same-input".into(),
                translation: translation.into(),
            })
            .unwrap();
    }

    assert_eq!(
        store.cached_translation("same-input").unwrap().as_deref(),
        Some("月光石")
    );
}

#[test]
fn persists_cache_in_a_file_backed_store() {
    let path = std::env::temp_dir().join(format!(
        "game-translator-cache-{}.sqlite3",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    ProjectStore::open(&path)
        .unwrap()
        .put_cache(&CacheEntry {
            input_fingerprint: "persistent".into(),
            translation: "译文".into(),
        })
        .unwrap();

    let reopened = ProjectStore::open(&path).unwrap();

    assert_eq!(
        reopened
            .cached_translation("persistent")
            .unwrap()
            .as_deref(),
        Some("译文")
    );
    let _ = std::fs::remove_file(path);
}
