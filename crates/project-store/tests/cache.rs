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
