mod cache;
mod jobs;
mod migrations;
mod projects;
mod segments;

use std::{error::Error, fmt};

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
