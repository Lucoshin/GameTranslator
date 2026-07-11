use rusqlite::{params, OptionalExtension};

use crate::{ProjectStore, StoreError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRecord {
    pub id: String,
    pub root: String,
}

impl ProjectStore {
    /// Inserts a project record.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the ID already exists or `SQLite` rejects the write.
    pub fn insert_project(&self, project: &ProjectRecord) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO projects (id, root) VALUES (?1, ?2)",
            params![project.id, project.root],
        )?;
        Ok(())
    }

    /// Looks up a project by its stable ID.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` cannot execute the query.
    pub fn project(&self, id: &str) -> Result<Option<ProjectRecord>, StoreError> {
        self.connection
            .query_row("SELECT id, root FROM projects WHERE id = ?1", [id], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    root: row.get(1)?,
                })
            })
            .optional()
            .map_err(Into::into)
    }
}
