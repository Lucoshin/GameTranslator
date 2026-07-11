use game_translator_project_store::{BatchRecord, ProjectRecord, ProjectStore, SegmentRecord};

#[test]
fn persists_projects_and_segments() {
    let store = ProjectStore::open_in_memory().unwrap();
    store
        .insert_project(&ProjectRecord {
            id: "project-1".into(),
            root: "D:/Games/Fixture".into(),
        })
        .unwrap();
    store
        .insert_segment(&SegmentRecord {
            id: "segment-1".into(),
            project_id: "project-1".into(),
            source: "月光石".into(),
        })
        .unwrap();

    assert_eq!(
        store.project("project-1").unwrap().unwrap().root,
        "D:/Games/Fixture"
    );
    assert_eq!(
        store.segment("segment-1").unwrap().unwrap().source,
        "月光石"
    );
}

#[test]
fn resumes_only_batches_that_were_not_completed() {
    let mut store = ProjectStore::open_in_memory().unwrap();
    store
        .insert_project(&ProjectRecord {
            id: "project-1".into(),
            root: "D:/Games/Fixture".into(),
        })
        .unwrap();
    store
        .create_job(
            "job-1",
            "project-1",
            &[
                BatchRecord {
                    id: "batch-1".into(),
                    input_fingerprint: "fingerprint-1".into(),
                },
                BatchRecord {
                    id: "batch-2".into(),
                    input_fingerprint: "fingerprint-2".into(),
                },
            ],
        )
        .unwrap();

    store.complete_batch("job-1", "batch-1").unwrap();

    assert_eq!(store.resumable_batch_ids("job-1").unwrap(), vec!["batch-2"]);
}
