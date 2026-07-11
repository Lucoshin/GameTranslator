use rusqlite::Connection;

use crate::StoreError;

pub(crate) fn migrate(connection: &Connection) -> Result<(), StoreError> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE projects (
            id TEXT PRIMARY KEY,
            root TEXT NOT NULL
        );

        CREATE TABLE segments (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            source TEXT NOT NULL,
            translation TEXT
        );

        CREATE TABLE jobs (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE batches (
            id TEXT NOT NULL,
            job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
            input_fingerprint TEXT NOT NULL,
            state TEXT NOT NULL CHECK (state IN ('pending', 'completed')),
            PRIMARY KEY (job_id, id)
        );

        CREATE TABLE translation_cache (
            input_fingerprint TEXT PRIMARY KEY,
            translation TEXT NOT NULL
        );

        CREATE TABLE glossary (
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            source_term TEXT NOT NULL,
            target_term TEXT NOT NULL,
            PRIMARY KEY (project_id, source_term)
        );

        CREATE TABLE translation_memory (
            input_fingerprint TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            target TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}
