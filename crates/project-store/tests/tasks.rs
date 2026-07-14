use game_translator_project_store::{ProjectStore, TaskRecord, TaskState};

#[test]
fn persists_and_recovers_an_incomplete_task_snapshot() {
    let path = std::env::temp_dir().join(format!(
        "game-translator-task-{}-{}.sqlite3",
        std::process::id(),
        1
    ));
    let _ = std::fs::remove_file(&path);
    {
        let store = ProjectStore::open(&path).unwrap();
        store
            .create_task(&TaskRecord::new("run-1", "C:/Games/Test", 12))
            .unwrap();
        store
            .update_task("run-1", TaskState::Running, 4, 1, Some("{\"items\":[]}"))
            .unwrap();
        store
            .update_task_progress("run-1", 6, 1, Some("{\"phase\":\"translating\"}"))
            .unwrap();
    }

    let store = ProjectStore::open(&path).unwrap();
    let tasks = store.resumable_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "run-1");
    assert_eq!(tasks[0].completed, 6);
    assert_eq!(tasks[0].failed, 1);
    assert_eq!(
        tasks[0].snapshot.as_deref(),
        Some("{\"phase\":\"translating\"}")
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn rejects_a_transition_out_of_a_terminal_state() {
    let store = ProjectStore::open_in_memory().unwrap();
    store
        .create_task(&TaskRecord::new("run-1", "C:/Games/Test", 1))
        .unwrap();
    store
        .update_task("run-1", TaskState::Running, 0, 0, None)
        .unwrap();
    store
        .update_task("run-1", TaskState::Completed, 1, 0, None)
        .unwrap();

    let error = store
        .update_task("run-1", TaskState::Running, 1, 0, None)
        .unwrap_err();
    assert!(error.to_string().contains("invalid task transition"));
    assert!(store.resumable_tasks().unwrap().is_empty());
}
