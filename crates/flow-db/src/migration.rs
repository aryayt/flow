use flow_core::Result;
use rusqlite::Connection;

const CURRENT_VERSION: i32 = 1;

/// Run database migrations to bring the schema up to date.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| {
            flow_core::FlowError::Database(format!("failed to get user_version: {e}"))
        })?;

    if version < 1 {
        migrate_v1(conn)?;
    }

    // Future migrations would go here
    // if version < 2 { migrate_v2(conn)?; }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Running migration v1: creating initial schema");

    // Create features table
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS features (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            priority INTEGER NOT NULL DEFAULT 0,
            category TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            steps TEXT NOT NULL DEFAULT '[]',
            passes INTEGER NOT NULL DEFAULT 0,
            in_progress INTEGER NOT NULL DEFAULT 0,
            dependencies TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        ",
        [],
    )
    .map_err(|e| {
        flow_core::FlowError::Database(format!("failed to create features table: {e}"))
    })?;

    // Create change_events table
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS change_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_id INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            field TEXT,
            old_value TEXT,
            new_value TEXT,
            agent TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        )
        ",
        [],
    )
    .map_err(|e| {
        flow_core::FlowError::Database(format!("failed to create change_events table: {e}"))
    })?;

    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS ix_feature_status ON features(passes, in_progress)",
        [],
    )
    .map_err(|e| {
        flow_core::FlowError::Database(format!("failed to create status index: {e}"))
    })?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS ix_feature_priority ON features(priority)",
        [],
    )
    .map_err(|e| {
        flow_core::FlowError::Database(format!("failed to create priority index: {e}"))
    })?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS ix_change_events_feature ON change_events(feature_id)",
        [],
    )
    .map_err(|e| {
        flow_core::FlowError::Database(format!("failed to create events index: {e}"))
    })?;

    // Update version
    conn.pragma_update(None, "user_version", CURRENT_VERSION)
        .map_err(|e| {
            flow_core::FlowError::Database(format!("failed to update user_version: {e}"))
        })?;

    tracing::info!("Migration v1 complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        let conn = Connection::open_in_memory().unwrap();

        // Initially version should be 0
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 0);

        // Run migrations
        run_migrations(&conn).unwrap();

        // Version should now be CURRENT_VERSION
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);

        // Tables should exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"features".to_string()));
        assert!(tables.contains(&"change_events".to_string()));

        // Running migrations again should be idempotent
        run_migrations(&conn).unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn test_feature_table_schema() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify we can insert and retrieve a feature
        conn.execute(
            r#"
            INSERT INTO features (name, description, priority, category, steps, dependencies)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                "Test Feature",
                "Test Description",
                1,
                "Test",
                "[]",
                "[1, 2]"
            ],
        )
        .unwrap();

        let (name, deps): (String, String) = conn
            .query_row("SELECT name, dependencies FROM features WHERE id = 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert_eq!(name, "Test Feature");
        assert_eq!(deps, "[1, 2]");
    }
}
