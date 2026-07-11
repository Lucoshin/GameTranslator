use rusqlite::{params, OptionalExtension};

use crate::{ProjectStore, StoreError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentRecord {
    pub id: String,
    pub project_id: String,
    pub source: String,
}

impl ProjectStore {
    /// Inserts a source segment for a project.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the project does not exist, the ID conflicts, or `SQLite` fails.
    pub fn insert_segment(&self, segment: &SegmentRecord) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO segments (id, project_id, source) VALUES (?1, ?2, ?3)",
            params![segment.id, segment.project_id, segment.source],
        )?;
        Ok(())
    }

    /// Looks up a source segment by its stable ID.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` cannot execute the query.
    pub fn segment(&self, id: &str) -> Result<Option<SegmentRecord>, StoreError> {
        self.connection
            .query_row(
                "SELECT id, project_id, source FROM segments WHERE id = ?1",
                [id],
                |row| {
                    Ok(SegmentRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        source: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }
}
