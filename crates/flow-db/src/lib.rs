//! `SQLite` database layer for Agent Flow feature management.
//!
//! Provides thread-safe database access with WAL mode for concurrent reads,
//! automatic schema migrations, and event logging for audit trails.
//!
//! # Architecture
//!
//! - Single write connection behind `Arc<Mutex<Connection>>` for thread safety
//! - Read-only connection spawning for parallel queries
//! - Automatic WAL mode and performance pragmas
//!
//! # Example
//!
//! ```rust,no_run
//! use flow_db::Database;
//!
//! let db = Database::open(std::path::Path::new("flow.db")).unwrap();
//! let features = flow_db::FeatureStore::get_all(&db.writer().lock().unwrap()).unwrap();
//! ```

pub mod event_log;
pub mod feature;
pub mod migration;
pub mod schema;
pub mod task_sync;

pub use feature::FeatureStore;

use flow_core::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Database handle with thread-safe write connection and ability to spawn read-only connections.
pub struct Database {
    writer: Arc<Mutex<Connection>>,
    db_path: std::path::PathBuf,
}

impl Database {
    /// Open a database at the specified path, creating it if necessary.
    /// Runs migrations and applies performance optimizations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| flow_core::FlowError::Database(format!("failed to open database: {e}")))?;

        schema::init_connection(&conn)?;
        migration::run_migrations(&conn)?;

        Ok(Self {
            writer: Arc::new(Mutex::new(conn)),
            db_path: path.to_path_buf(),
        })
    }

    /// Open an in-memory database for testing.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            flow_core::FlowError::Database(format!("failed to open in-memory database: {e}"))
        })?;

        schema::init_connection(&conn)?;
        migration::run_migrations(&conn)?;

        Ok(Self {
            writer: Arc::new(Mutex::new(conn)),
            db_path: std::path::PathBuf::from(":memory:"),
        })
    }

    /// Get the shared writer connection (behind a mutex).
    pub const fn writer(&self) -> &Arc<Mutex<Connection>> {
        &self.writer
    }

    /// Create a new read-only connection to the same database.
    pub fn new_reader(&self) -> Result<Connection> {
        if self.db_path.to_str() == Some(":memory:") {
            return Err(flow_core::FlowError::Database(
                "cannot create reader for in-memory database".to_string(),
            ));
        }

        let conn =
            Connection::open_with_flags(&self.db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| {
                    flow_core::FlowError::Database(format!(
                        "failed to open read-only connection: {e}"
                    ))
                })?;

        schema::init_connection(&conn)?;
        Ok(conn)
    }
}

#[cfg(test)]
#[allow(clippy::significant_drop_tightening)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        // Verify tables exist
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"features".to_string()));
        assert!(tables.contains(&"change_events".to_string()));
    }

    #[test]
    fn test_concurrent_access() {
        let db = Database::open_in_memory().unwrap();

        // Writer creates a feature
        {
            let conn = db.writer().lock().unwrap();
            FeatureStore::create(
                &conn,
                &flow_core::CreateFeatureInput {
                    name: "Test Feature".to_string(),
                    description: "Test".to_string(),
                    priority: Some(1),
                    category: "Test".to_string(),
                    steps: vec![],
                    dependencies: vec![],
                },
            )
            .unwrap();
        }

        // Reader can read it
        {
            let conn = db.writer().lock().unwrap();
            let features = FeatureStore::get_all(&conn).unwrap();
            assert_eq!(features.len(), 1);
            assert_eq!(features[0].name, "Test Feature");
        }
    }
}
