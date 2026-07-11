use rusqlite::params;

use crate::{ProjectStore, StoreError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchRecord {
    pub id: String,
    pub input_fingerprint: String,
}

impl ProjectStore {
    /// Creates a translation job and all of its pending batches atomically.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the project is missing, an ID conflicts, or the transaction fails.
    pub fn create_job(
        &mut self,
        job_id: &str,
        project_id: &str,
        batches: &[BatchRecord],
    ) -> Result<(), StoreError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO jobs (id, project_id) VALUES (?1, ?2)",
            params![job_id, project_id],
        )?;
        for batch in batches {
            transaction.execute(
                "INSERT INTO batches (id, job_id, input_fingerprint, state)
                 VALUES (?1, ?2, ?3, 'pending')",
                params![batch.id, job_id, batch.input_fingerprint],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Marks one batch complete.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the batch does not exist or `SQLite` rejects the update.
    pub fn complete_batch(&mut self, job_id: &str, batch_id: &str) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE batches SET state = 'completed' WHERE job_id = ?1 AND id = ?2",
            params![job_id, batch_id],
        )?;
        if changed == 0 {
            return Err(StoreError(format!(
                "batch {batch_id} does not exist in job {job_id}"
            )));
        }
        Ok(())
    }

    /// Returns pending batch IDs in insertion order for task recovery.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when `SQLite` cannot execute the query.
    pub fn resumable_batch_ids(&self, job_id: &str) -> Result<Vec<String>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id FROM batches WHERE job_id = ?1 AND state = 'pending' ORDER BY rowid",
        )?;
        let rows = statement.query_map([job_id], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}
