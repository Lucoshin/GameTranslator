mod cache;
mod jobs;
mod migrations;
mod projects;
mod segments;

use std::{error::Error, fmt, path::Path};

use rusqlite::Connection;

pub use cache::CacheEntry;
pub use jobs::BatchRecord;
pub use projects::ProjectRecord;
pub use segments::SegmentRecord;

#[derive(Debug)]
pub struct StoreError(String);

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self(error.to_string())
    }
}

pub struct ProjectStore {
    connection: Connection,
}

impl ProjectStore {
    /// Opens or creates a persistent `SQLite` store and applies migrations.
    ///
    /// # Errors
    /// Returns [`StoreError`] when the directory, database, or schema cannot be initialized.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| StoreError(error.to_string()))?;
        }
        let connection = Connection::open(path)?;
        migrations::migrate(&connection)?;
        Ok(Self { connection })
    }

    /// Creates an isolated in-memory store and applies all migrations.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` cannot initialize the schema.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let connection = Connection::open_in_memory()?;
        migrations::migrate(&connection)?;
        Ok(Self { connection })
    }
}
