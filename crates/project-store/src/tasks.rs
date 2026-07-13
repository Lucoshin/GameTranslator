use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, OptionalExtension};

use crate::{ProjectStore, StoreError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(StoreError(format!("unknown task state: {value}"))),
        }
    }

    const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (
                Self::Pending | Self::Paused,
                Self::Running | Self::Cancelled
            ) | (
                Self::Running,
                Self::Paused | Self::Completed | Self::Failed | Self::Cancelled
            )
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub project_path: String,
    pub state: TaskState,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub snapshot: Option<String>,
    pub updated_at_unix_ms: u64,
}

impl TaskRecord {
    #[must_use]
    pub fn new(id: impl Into<String>, project_path: impl Into<String>, total: usize) -> Self {
        Self {
            id: id.into(),
            project_path: project_path.into(),
            state: TaskState::Pending,
            total,
            completed: 0,
            failed: 0,
            snapshot: None,
            updated_at_unix_ms: 0,
        }
    }
}

impl ProjectStore {
    /// # Errors
    /// Returns an error when the task already exists or cannot be persisted.
    pub fn create_task(&self, task: &TaskRecord) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO translation_tasks
             (id, project_path, state, total, completed, failed, snapshot, updated_at_unix_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task.id,
                task.project_path,
                task.state.as_str(),
                to_i64(task.total)?,
                to_i64(task.completed)?,
                to_i64(task.failed)?,
                task.snapshot,
                to_i64(now_unix_ms()?)?
            ],
        )?;
        Ok(())
    }

    /// # Errors
    /// Returns an error for missing tasks, invalid state transitions, or storage failures.
    pub fn update_task(
        &self,
        id: &str,
        next: TaskState,
        completed: usize,
        failed: usize,
        snapshot: Option<&str>,
    ) -> Result<(), StoreError> {
        let current = self
            .connection
            .query_row(
                "SELECT state FROM translation_tasks WHERE id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| StoreError(format!("task not found: {id}")))?;
        let current = TaskState::parse(&current)?;
        if !current.can_transition_to(next) {
            return Err(StoreError(format!(
                "invalid task transition: {} -> {}",
                current.as_str(),
                next.as_str()
            )));
        }
        self.connection.execute(
            "UPDATE translation_tasks
             SET state = ?2, completed = ?3, failed = ?4,
                 snapshot = COALESCE(?5, snapshot), updated_at_unix_ms = ?6
             WHERE id = ?1",
            params![
                id,
                next.as_str(),
                to_i64(completed)?,
                to_i64(failed)?,
                snapshot,
                to_i64(now_unix_ms()?)?
            ],
        )?;
        Ok(())
    }

    /// Updates the recoverable snapshot without changing the task state.
    ///
    /// # Errors
    /// Returns an error when the task is missing, terminal, or cannot be persisted.
    pub fn update_task_progress(
        &self,
        id: &str,
        completed: usize,
        failed: usize,
        snapshot: Option<&str>,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE translation_tasks
             SET completed = ?2, failed = ?3, snapshot = COALESCE(?4, snapshot),
                 updated_at_unix_ms = ?5
             WHERE id = ?1 AND state IN ('pending', 'running', 'paused')",
            params![
                id,
                to_i64(completed)?,
                to_i64(failed)?,
                snapshot,
                to_i64(now_unix_ms()?)?
            ],
        )?;
        if changed == 0 {
            return Err(StoreError(format!("task not found or not resumable: {id}")));
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error when resumable task snapshots cannot be read.
    pub fn resumable_tasks(&self) -> Result<Vec<TaskRecord>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_path, state, total, completed, failed, snapshot, updated_at_unix_ms
             FROM translation_tasks
             WHERE state IN ('pending', 'running', 'paused')
             ORDER BY updated_at_unix_ms DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })?;
        rows.map(|row| {
            let (id, project_path, state, total, completed, failed, snapshot, updated_at_unix_ms) =
                row?;
            Ok(TaskRecord {
                id,
                project_path,
                state: TaskState::parse(&state)?,
                total: usize::try_from(total).map_err(|error| StoreError(error.to_string()))?,
                completed: usize::try_from(completed)
                    .map_err(|error| StoreError(error.to_string()))?,
                failed: usize::try_from(failed).map_err(|error| StoreError(error.to_string()))?,
                snapshot,
                updated_at_unix_ms: u64::try_from(updated_at_unix_ms)
                    .map_err(|error| StoreError(error.to_string()))?,
            })
        })
        .collect()
    }
}

fn now_unix_ms() -> Result<u64, StoreError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StoreError(error.to_string()))?
        .as_millis();
    u64::try_from(millis).map_err(|error| StoreError(error.to_string()))
}

fn to_i64<T>(value: T) -> Result<i64, StoreError>
where
    T: TryInto<i64>,
    T::Error: std::fmt::Display,
{
    value
        .try_into()
        .map_err(|error| StoreError(error.to_string()))
}
