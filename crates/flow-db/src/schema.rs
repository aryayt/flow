use flow_core::Result;
use rusqlite::Connection;
use std::time::Duration;

/// Initialize a database connection with performance optimizations.
pub fn init_connection(conn: &Connection) -> Result<()> {
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| flow_core::FlowError::Database(format!("failed to set WAL mode: {e}")))?;

    // Synchronous = NORMAL for better performance (still safe with WAL)
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| {
            flow_core::FlowError::Database(format!("failed to set synchronous mode: {e}"))
        })?;

    // Cache size: 64MB (negative value means kibibytes)
    conn.pragma_update(None, "cache_size", -64000)
        .map_err(|e| flow_core::FlowError::Database(format!("failed to set cache size: {e}")))?;

    // Store temp tables in memory for speed
    conn.pragma_update(None, "temp_store", "MEMORY")
        .map_err(|e| flow_core::FlowError::Database(format!("failed to set temp_store: {e}")))?;

    // Enable memory-mapped I/O (30GB max)
    conn.pragma_update(None, "mmap_size", 30_000_000_000_i64)
        .map_err(|e| flow_core::FlowError::Database(format!("failed to set mmap_size: {e}")))?;

    // Set busy handler: retry up to 100 times with 10ms sleep
    conn.busy_timeout(Duration::from_millis(1000))
        .map_err(|e| flow_core::FlowError::Database(format!("failed to set busy timeout: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_connection() {
        let conn = Connection::open_in_memory().unwrap();
        init_connection(&conn).unwrap();

        // Note: In-memory databases use MEMORY journal mode, not WAL
        // WAL is only supported for file-based databases
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_uppercase(), "MEMORY");

        // Verify cache size
        let cache_size: i32 = conn
            .pragma_query_value(None, "cache_size", |row| row.get(0))
            .unwrap();
        assert_eq!(cache_size, -64000);
    }
}
