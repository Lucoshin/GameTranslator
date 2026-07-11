use rusqlite::{params, OptionalExtension};

use crate::{ProjectStore, StoreError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry {
    pub input_fingerprint: String,
    pub translation: String,
}

impl ProjectStore {
    /// Inserts or replaces the translation for an exact input fingerprint.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` rejects the write.
    pub fn put_cache(&self, entry: &CacheEntry) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO translation_cache (input_fingerprint, translation)
             VALUES (?1, ?2)
             ON CONFLICT(input_fingerprint) DO UPDATE SET translation = excluded.translation",
            params![entry.input_fingerprint, entry.translation],
        )?;
        Ok(())
    }

    /// Looks up a translation by its exact input fingerprint.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` cannot execute the query.
    pub fn cached_translation(&self, fingerprint: &str) -> Result<Option<String>, StoreError> {
        self.connection
            .query_row(
                "SELECT translation FROM translation_cache WHERE input_fingerprint = ?1",
                [fingerprint],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }
}
