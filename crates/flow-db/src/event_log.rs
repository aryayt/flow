use flow_core::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// A change event logged for a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: i64,
    pub feature_id: i64,
    pub event_type: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub agent: Option<String>,
    pub source: String,
    pub created_at: String,
}

/// Log a change event for a feature.
pub fn log_event(
    conn: &Connection,
    feature_id: i64,
    event_type: &str,
    field: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    agent: Option<&str>,
    source: &str,
) -> Result<()> {
    conn.execute(
        r"
        INSERT INTO change_events (feature_id, event_type, field, old_value, new_value, agent, source)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        rusqlite::params![
            feature_id,
            event_type,
            field,
            old_value,
            new_value,
            agent,
            source,
        ],
    )
    .map_err(|e| flow_core::FlowError::Database(format!("log event failed: {e}")))?;

    Ok(())
}

/// Get all change events for a feature.
pub fn get_events(conn: &Connection, feature_id: i64) -> Result<Vec<ChangeEvent>> {
    let mut stmt = conn
        .prepare(
            r"
            SELECT id, feature_id, event_type, field, old_value, new_value, agent, source, created_at
            FROM change_events
            WHERE feature_id = ?
            ORDER BY created_at ASC
            ",
        )
        .map_err(|e| flow_core::FlowError::Database(format!("prepare failed: {e}")))?;

    let events = stmt
        .query_map([feature_id], |row| {
            Ok(ChangeEvent {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                event_type: row.get(2)?,
                field: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                agent: row.get(6)?,
                source: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| flow_core::FlowError::Database(format!("query failed: {e}")))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| flow_core::FlowError::Database(format!("row parse failed: {e}")))?;

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureStore;
    use crate::Database;

    #[test]
    fn test_log_and_get_events() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        // Create a feature first
        let feature = FeatureStore::create(
            &conn,
            &flow_core::CreateFeatureInput {
                name: "Test Feature".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        // Log some events
        log_event(
            &conn,
            feature.id,
            "status_change",
            Some("status"),
            Some("pending"),
            Some("in_progress"),
            Some("agent-1"),
            "api",
        )
        .unwrap();

        log_event(
            &conn,
            feature.id,
            "status_change",
            Some("status"),
            Some("in_progress"),
            Some("completed"),
            Some("agent-1"),
            "api",
        )
        .unwrap();

        // Retrieve events
        let events = get_events(&conn, feature.id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "status_change");
        assert_eq!(events[0].old_value, Some("pending".to_string()));
        assert_eq!(events[0].new_value, Some("in_progress".to_string()));
        assert_eq!(events[1].new_value, Some("completed".to_string()));
    }

    #[test]
    fn test_events_for_nonexistent_feature() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let events = get_events(&conn, 999).unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_with_optional_fields() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let feature = FeatureStore::create(
            &conn,
            &flow_core::CreateFeatureInput {
                name: "Test".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        // Log event with minimal fields
        log_event(
            &conn,
            feature.id,
            "comment",
            None,
            None,
            None,
            None,
            "web",
        )
        .unwrap();

        let events = get_events(&conn, feature.id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "comment");
        assert!(events[0].field.is_none());
        assert!(events[0].agent.is_none());
    }
}
