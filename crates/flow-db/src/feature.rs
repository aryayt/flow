use flow_core::{CreateFeatureInput, DependencyRef, Feature, FeatureGraphNode, FeatureStats, Result};
use rusqlite::{Connection, OptionalExtension, Row};
use std::collections::{HashMap, HashSet};

pub struct FeatureStore;

impl FeatureStore {
    /// Create a new feature.
    pub fn create(conn: &Connection, input: &CreateFeatureInput) -> Result<Feature> {
        let priority = input.priority.map_or_else(|| {
                // Auto-assign priority as MAX+1
                let max_priority: Option<i32> = conn
                    .query_row("SELECT MAX(priority) FROM features", [], |row| row.get(0))
                    .ok()
                    .flatten();
                max_priority.unwrap_or(0) + 1
            }, |p| p);

        // Validate dependencies exist
        for dep_ref in &input.dependencies {
            if let DependencyRef::Id(dep_id) = dep_ref {
                let exists: bool = conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM features WHERE id = ?)",
                        [dep_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| {
                        flow_core::FlowError::Database(format!("dependency check failed: {e}"))
                    })?;
                if !exists {
                    return Err(flow_core::FlowError::NotFound(format!(
                        "dependency {dep_id} not found"
                    )));
                }
            }
        }

        let deps_json = serde_json::to_string(
            &input
                .dependencies
                .iter()
                .filter_map(|d| match d {
                    DependencyRef::Id(id) => Some(*id),
                    DependencyRef::Index { .. } => None,
                })
                .collect::<Vec<_>>(),
        )?;
        let steps_json = serde_json::to_string(&input.steps)?;

        conn.execute(
            r"
            INSERT INTO features (name, description, priority, category, steps, dependencies)
            VALUES (?, ?, ?, ?, ?, ?)
            ",
            rusqlite::params![
                &input.name,
                &input.description,
                priority,
                &input.category,
                steps_json,
                deps_json,
            ],
        )
        .map_err(|e| flow_core::FlowError::Database(format!("insert failed: {e}")))?;

        let id = conn.last_insert_rowid();
        Self::get_by_id(conn, id)?.ok_or_else(|| {
            flow_core::FlowError::Database("feature not found after insert".to_string())
        })
    }

    /// Create multiple features in a single transaction, resolving index-based dependencies.
    pub fn create_bulk(conn: &Connection, inputs: &[CreateFeatureInput]) -> Result<Vec<Feature>> {
        if inputs.is_empty() {
            return Ok(vec![]);
        }

        // Validate all inputs first (before starting transaction)
        for (idx, input) in inputs.iter().enumerate() {
            for dep_ref in &input.dependencies {
                match dep_ref {
                    DependencyRef::Id(dep_id) => {
                        let exists: bool = conn
                            .query_row(
                                "SELECT EXISTS(SELECT 1 FROM features WHERE id = ?)",
                                [dep_id],
                                |row| row.get(0),
                            )
                            .map_err(|e| {
                                flow_core::FlowError::Database(format!(
                                    "dependency check failed: {e}"
                                ))
                            })?;
                        if !exists {
                            return Err(flow_core::FlowError::NotFound(format!(
                                "dependency {dep_id} not found for feature at index {idx}"
                            )));
                        }
                    }
                    DependencyRef::Index { index } => {
                        if *index >= inputs.len() {
                            return Err(flow_core::FlowError::BadRequest(format!(
                                "index {index} out of bounds for feature at index {idx}"
                            )));
                        }
                        // Reject self-references
                        if *index == idx {
                            return Err(flow_core::FlowError::BadRequest(format!(
                                "feature at index {idx} cannot depend on itself"
                            )));
                        }
                    }
                }
            }
        }

        // Begin transaction for atomicity
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| flow_core::FlowError::Database(format!("begin transaction failed: {e}")))?;

        let result = (|| -> Result<Vec<Feature>> {
            // Get starting priority
            let max_priority: Option<i32> = conn
                .query_row("SELECT MAX(priority) FROM features", [], |row| row.get(0))
                .ok()
                .flatten();
            let mut next_priority = max_priority.unwrap_or(0) + 1;

            // Insert all features, tracking their IDs
            let mut feature_ids = Vec::new();
            for input in inputs {
                let priority = input.priority.unwrap_or_else(|| {
                    let p = next_priority;
                    next_priority += 1;
                    p
                });

                let steps_json = serde_json::to_string(&input.steps)?;
                let deps_json = "[]"; // Temporarily empty, will update after

                conn.execute(
                    r"
                    INSERT INTO features (name, description, priority, category, steps, dependencies)
                    VALUES (?, ?, ?, ?, ?, ?)
                    ",
                    rusqlite::params![
                        &input.name,
                        &input.description,
                        priority,
                        &input.category,
                        steps_json,
                        deps_json,
                    ],
                )
                .map_err(|e| flow_core::FlowError::Database(format!("bulk insert failed: {e}")))?;

                feature_ids.push(conn.last_insert_rowid());
            }

            // Resolve index-based dependencies to real IDs and update
            for (idx, input) in inputs.iter().enumerate() {
                let feature_id = feature_ids[idx];
                let mut resolved_deps = Vec::new();

                for dep_ref in &input.dependencies {
                    match dep_ref {
                        DependencyRef::Id(id) => resolved_deps.push(*id),
                        DependencyRef::Index { index } => resolved_deps.push(feature_ids[*index]),
                    }
                }

                if !resolved_deps.is_empty() {
                    let deps_json = serde_json::to_string(&resolved_deps)?;
                    conn.execute(
                        "UPDATE features SET dependencies = ? WHERE id = ?",
                        rusqlite::params![deps_json, feature_id],
                    )
                    .map_err(|e| {
                        flow_core::FlowError::Database(format!("dependency update failed: {e}"))
                    })?;
                }
            }

            // Fetch and return all created features
            feature_ids
                .into_iter()
                .map(|id| {
                    Self::get_by_id(conn, id)?
                        .ok_or_else(|| flow_core::FlowError::Database("feature disappeared".to_string()))
                })
                .collect()
        })();

        match result {
            Ok(features) => {
                conn.execute_batch("COMMIT")
                    .map_err(|e| flow_core::FlowError::Database(format!("commit failed: {e}")))?;
                Ok(features)
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Get a feature by ID.
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Feature>> {
        let result = conn
            .query_row(
                "SELECT * FROM features WHERE id = ?",
                [id],
                row_to_feature,
            )
            .optional()
            .map_err(|e| flow_core::FlowError::Database(format!("query failed: {e}")))?;
        Ok(result)
    }

    /// Get all features.
    pub fn get_all(conn: &Connection) -> Result<Vec<Feature>> {
        let mut stmt = conn
            .prepare("SELECT * FROM features ORDER BY priority ASC")
            .map_err(|e| flow_core::FlowError::Database(format!("prepare failed: {e}")))?;

        let features = stmt
            .query_map([], row_to_feature)
            .map_err(|e| flow_core::FlowError::Database(format!("query failed: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| flow_core::FlowError::Database(format!("row parse failed: {e}")))?;

        Ok(features)
    }

    /// Get feature statistics.
    pub fn get_stats(conn: &Connection) -> Result<FeatureStats> {
        let (total, passing, in_progress): (usize, usize, usize) = conn
            .query_row(
                r"
                SELECT
                    COUNT(*),
                    SUM(CASE WHEN passes = 1 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN in_progress = 1 THEN 1 ELSE 0 END)
                FROM features
                ",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| flow_core::FlowError::Database(format!("stats query failed: {e}")))?;

        let failing = total.saturating_sub(passing).saturating_sub(in_progress);

        // Count blocked features (those with at least one failing dependency)
        let all_features = Self::get_all(conn)?;
        let blocked = all_features
            .iter()
            .filter(|f| {
                f.dependencies.iter().any(|dep_id| {
                    all_features
                        .iter()
                        .find(|d| d.id == *dep_id)
                        .is_some_and(|d| !d.passes)
                })
            })
            .count();

        Ok(FeatureStats {
            total,
            passing,
            failing,
            in_progress,
            blocked,
        })
    }

    /// Get features that are ready to work on (not in progress, not passing, all deps pass).
    pub fn get_ready(conn: &Connection) -> Result<Vec<Feature>> {
        let all_features = Self::get_all(conn)?;
        let passing_ids: HashSet<i64> = all_features
            .iter()
            .filter(|f| f.passes)
            .map(|f| f.id)
            .collect();

        let ready: Vec<Feature> = all_features
            .into_iter()
            .filter(|f| {
                !f.in_progress
                    && !f.passes
                    && f.dependencies.iter().all(|dep_id| passing_ids.contains(dep_id))
            })
            .collect();

        Ok(ready)
    }

    /// Get features that are blocked by failing dependencies.
    pub fn get_blocked(conn: &Connection) -> Result<Vec<Feature>> {
        let all_features = Self::get_all(conn)?;
        let passing_ids: HashSet<i64> = all_features
            .iter()
            .filter(|f| f.passes)
            .map(|f| f.id)
            .collect();

        let blocked: Vec<Feature> = all_features
            .into_iter()
            .filter(|f| {
                !f.dependencies.is_empty()
                    && f.dependencies
                        .iter()
                        .any(|dep_id| !passing_ids.contains(dep_id))
            })
            .collect();

        Ok(blocked)
    }

    /// Atomically claim a feature and mark it in progress.
    pub fn claim_and_get(conn: &Connection, id: i64) -> Result<Feature> {
        let rows_affected = conn
            .execute(
                r"
                UPDATE features
                SET in_progress = 1, updated_at = datetime('now')
                WHERE id = ? AND in_progress = 0 AND passes = 0
                ",
                [id],
            )
            .map_err(|e| flow_core::FlowError::Database(format!("claim failed: {e}")))?;

        if rows_affected == 0 {
            return Err(flow_core::FlowError::Conflict(format!(
                "feature {id} is already claimed or completed"
            )));
        }

        Self::get_by_id(conn, id)?
            .ok_or_else(|| flow_core::FlowError::NotFound(format!("feature {id} not found")))
    }

    /// Mark a feature as passing.
    pub fn mark_passing(conn: &Connection, id: i64) -> Result<()> {
        let rows = conn
            .execute(
                "UPDATE features SET passes = 1, in_progress = 0, updated_at = datetime('now') WHERE id = ?",
                [id],
            )
            .map_err(|e| flow_core::FlowError::Database(format!("update failed: {e}")))?;
        if rows == 0 {
            return Err(flow_core::FlowError::NotFound(format!(
                "feature {id} not found"
            )));
        }
        Ok(())
    }

    /// Mark a feature as failing.
    pub fn mark_failing(conn: &Connection, id: i64) -> Result<()> {
        let rows = conn
            .execute(
                "UPDATE features SET passes = 0, in_progress = 0, updated_at = datetime('now') WHERE id = ?",
                [id],
            )
            .map_err(|e| flow_core::FlowError::Database(format!("update failed: {e}")))?;
        if rows == 0 {
            return Err(flow_core::FlowError::NotFound(format!(
                "feature {id} not found"
            )));
        }
        Ok(())
    }

    /// Mark a feature as in progress (atomic).
    pub fn mark_in_progress(conn: &Connection, id: i64) -> Result<()> {
        let rows = conn
            .execute(
                "UPDATE features SET in_progress = 1, updated_at = datetime('now') WHERE id = ? AND in_progress = 0",
                [id],
            )
            .map_err(|e| flow_core::FlowError::Database(format!("update failed: {e}")))?;

        if rows == 0 {
            return Err(flow_core::FlowError::Conflict(format!(
                "feature {id} is already in progress"
            )));
        }
        Ok(())
    }

    /// Clear in-progress flag.
    pub fn clear_in_progress(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE features SET in_progress = 0, updated_at = datetime('now') WHERE id = ?",
            [id],
        )
        .map_err(|e| flow_core::FlowError::Database(format!("update failed: {e}")))?;
        Ok(())
    }

    /// Skip a feature by moving it to the end of the priority queue.
    pub fn skip(conn: &Connection, id: i64) -> Result<()> {
        let max_priority: Option<i32> = conn
            .query_row("SELECT MAX(priority) FROM features", [], |row| row.get(0))
            .ok()
            .flatten();
        let new_priority = max_priority.unwrap_or(0) + 1;

        conn.execute(
            "UPDATE features SET priority = ?, updated_at = datetime('now') WHERE id = ?",
            rusqlite::params![new_priority, id],
        )
        .map_err(|e| flow_core::FlowError::Database(format!("skip failed: {e}")))?;
        Ok(())
    }

    /// Add a dependency to a feature.
    /// Uses SAVEPOINT for atomic read-modify-write to prevent TOCTOU races.
    pub fn add_dependency(conn: &Connection, feature_id: i64, dep_id: i64) -> Result<()> {
        conn.execute_batch("SAVEPOINT add_dep")
            .map_err(|e| flow_core::FlowError::Database(format!("savepoint failed: {e}")))?;

        let result = (|| -> Result<()> {
            // Verify dependency exists
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM features WHERE id = ?)",
                    [dep_id],
                    |row| row.get(0),
                )
                .map_err(|e| flow_core::FlowError::Database(format!("exists check failed: {e}")))?;

            if !exists {
                return Err(flow_core::FlowError::NotFound(format!(
                    "dependency {dep_id} not found"
                )));
            }

            // Get current dependencies (within the savepoint for consistency)
            let deps_json: String = conn
                .query_row(
                    "SELECT dependencies FROM features WHERE id = ?",
                    [feature_id],
                    |row| row.get(0),
                )
                .map_err(|e| flow_core::FlowError::Database(format!("query failed: {e}")))?;

            let mut deps: Vec<i64> = serde_json::from_str(&deps_json)?;

            if !deps.contains(&dep_id) {
                deps.push(dep_id);
                let new_deps_json = serde_json::to_string(&deps)?;
                conn.execute(
                    "UPDATE features SET dependencies = ?, updated_at = datetime('now') WHERE id = ?",
                    rusqlite::params![new_deps_json, feature_id],
                )
                .map_err(|e| {
                    flow_core::FlowError::Database(format!("dependency add failed: {e}"))
                })?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("RELEASE SAVEPOINT add_dep")
                    .map_err(|e| flow_core::FlowError::Database(format!("release failed: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK TO SAVEPOINT add_dep");
                Err(e)
            }
        }
    }

    /// Remove a dependency from a feature.
    /// Uses SAVEPOINT for atomic read-modify-write to prevent TOCTOU races.
    pub fn remove_dependency(conn: &Connection, feature_id: i64, dep_id: i64) -> Result<()> {
        conn.execute_batch("SAVEPOINT rm_dep")
            .map_err(|e| flow_core::FlowError::Database(format!("savepoint failed: {e}")))?;

        let result = (|| -> Result<()> {
            let deps_json: String = conn
                .query_row(
                    "SELECT dependencies FROM features WHERE id = ?",
                    [feature_id],
                    |row| row.get(0),
                )
                .map_err(|e| flow_core::FlowError::Database(format!("query failed: {e}")))?;

            let mut deps: Vec<i64> = serde_json::from_str(&deps_json)?;
            deps.retain(|&id| id != dep_id);

            let new_deps_json = serde_json::to_string(&deps)?;
            conn.execute(
                "UPDATE features SET dependencies = ?, updated_at = datetime('now') WHERE id = ?",
                rusqlite::params![new_deps_json, feature_id],
            )
            .map_err(|e| {
                flow_core::FlowError::Database(format!("dependency remove failed: {e}"))
            })?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("RELEASE SAVEPOINT rm_dep")
                    .map_err(|e| flow_core::FlowError::Database(format!("release failed: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK TO SAVEPOINT rm_dep");
                Err(e)
            }
        }
    }

    /// Set all dependencies for a feature (replaces existing).
    /// Uses SAVEPOINT for atomic validation + update.
    pub fn set_dependencies(conn: &Connection, feature_id: i64, dep_ids: &[i64]) -> Result<()> {
        conn.execute_batch("SAVEPOINT set_deps")
            .map_err(|e| flow_core::FlowError::Database(format!("savepoint failed: {e}")))?;

        let result = (|| -> Result<()> {
            // Verify all dependencies exist
            for dep_id in dep_ids {
                let exists: bool = conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM features WHERE id = ?)",
                        [dep_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| {
                        flow_core::FlowError::Database(format!("exists check failed: {e}"))
                    })?;

                if !exists {
                    return Err(flow_core::FlowError::NotFound(format!(
                        "dependency {dep_id} not found"
                    )));
                }
            }

            let deps_json = serde_json::to_string(dep_ids)?;
            conn.execute(
                "UPDATE features SET dependencies = ?, updated_at = datetime('now') WHERE id = ?",
                rusqlite::params![deps_json, feature_id],
            )
            .map_err(|e| flow_core::FlowError::Database(format!("set dependencies failed: {e}")))?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("RELEASE SAVEPOINT set_deps")
                    .map_err(|e| flow_core::FlowError::Database(format!("release failed: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK TO SAVEPOINT set_deps");
                Err(e)
            }
        }
    }

    /// Get the complete dependency graph with dependents computed.
    pub fn get_graph(conn: &Connection) -> Result<Vec<FeatureGraphNode>> {
        let features = Self::get_all(conn)?;
        let passing_ids: HashSet<i64> = features
            .iter()
            .filter(|f| f.passes)
            .map(|f| f.id)
            .collect();

        // Build reverse dependency map (dependents)
        let mut dependents_map: HashMap<i64, Vec<i64>> = HashMap::new();
        for feature in &features {
            for dep_id in &feature.dependencies {
                dependents_map
                    .entry(*dep_id)
                    .or_default()
                    .push(feature.id);
            }
        }

        let graph: Vec<FeatureGraphNode> = features
            .into_iter()
            .map(|f| {
                let blocked = !f.dependencies.is_empty()
                    && f.dependencies
                        .iter()
                        .any(|dep_id| !passing_ids.contains(dep_id));

                FeatureGraphNode {
                    id: f.id,
                    name: f.name,
                    category: f.category,
                    priority: f.priority,
                    passes: f.passes,
                    in_progress: f.in_progress,
                    blocked,
                    dependencies: f.dependencies,
                    dependents: dependents_map.get(&f.id).cloned().unwrap_or_default(),
                }
            })
            .collect();

        Ok(graph)
    }
}

/// Helper to convert a database row to a Feature.
fn row_to_feature(row: &Row) -> rusqlite::Result<Feature> {
    let steps_json: String = row.get("steps")?;
    let steps: Vec<String> = serde_json::from_str(&steps_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;

    let deps_json: String = row.get("dependencies")?;
    let dependencies: Vec<i64> = serde_json::from_str(&deps_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;

    let passes_int: i32 = row.get("passes")?;
    let in_progress_int: i32 = row.get("in_progress")?;

    Ok(Feature {
        id: row.get("id")?,
        priority: row.get("priority")?,
        category: row.get("category")?,
        name: row.get("name")?,
        description: row.get("description")?,
        steps,
        passes: passes_int != 0,
        in_progress: in_progress_int != 0,
        dependencies,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    #[test]
    fn test_create_and_get() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let feature = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "Test Feature".to_string(),
                description: "Test Description".to_string(),
                priority: Some(1),
                category: "Test".to_string(),
                steps: vec!["Step 1".to_string(), "Step 2".to_string()],
                dependencies: vec![],
            },
        )
        .unwrap();

        assert_eq!(feature.id, 1);
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.priority, 1);
        assert_eq!(feature.steps.len(), 2);

        let retrieved = FeatureStore::get_by_id(&conn, 1).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Feature");
    }

    #[test]
    fn test_bulk_create_with_index_dependencies() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let inputs = vec![
            CreateFeatureInput {
                name: "Feature A".to_string(),
                description: "".to_string(),
                priority: None,
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
            CreateFeatureInput {
                name: "Feature B".to_string(),
                description: "".to_string(),
                priority: None,
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![DependencyRef::Index { index: 0 }],
            },
            CreateFeatureInput {
                name: "Feature C".to_string(),
                description: "".to_string(),
                priority: None,
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![DependencyRef::Index { index: 0 }, DependencyRef::Index { index: 1 }],
            },
        ];

        let features = FeatureStore::create_bulk(&conn, &inputs).unwrap();
        assert_eq!(features.len(), 3);

        // Feature B should depend on Feature A
        assert_eq!(features[1].dependencies, vec![features[0].id]);

        // Feature C should depend on both A and B
        assert_eq!(
            features[2].dependencies,
            vec![features[0].id, features[1].id]
        );
    }

    #[test]
    fn test_claim_and_get() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let feature = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "Claimable".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        // First claim should succeed
        let claimed = FeatureStore::claim_and_get(&conn, feature.id).unwrap();
        assert!(claimed.in_progress);

        // Second claim should fail
        let result = FeatureStore::claim_and_get(&conn, feature.id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), flow_core::FlowError::Conflict(_)));
    }

    #[test]
    fn test_mark_passing_failing() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let feature = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "Test".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        // Mark passing
        FeatureStore::mark_passing(&conn, feature.id).unwrap();
        let updated = FeatureStore::get_by_id(&conn, feature.id).unwrap().unwrap();
        assert!(updated.passes);
        assert!(!updated.in_progress);

        // Mark failing
        FeatureStore::mark_failing(&conn, feature.id).unwrap();
        let updated = FeatureStore::get_by_id(&conn, feature.id).unwrap().unwrap();
        assert!(!updated.passes);
        assert!(!updated.in_progress);
    }

    #[test]
    fn test_dependencies() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let f1 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F1".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        let f2 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F2".to_string(),
                description: "".to_string(),
                priority: Some(2),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        // Add dependency
        FeatureStore::add_dependency(&conn, f2.id, f1.id).unwrap();
        let updated = FeatureStore::get_by_id(&conn, f2.id).unwrap().unwrap();
        assert_eq!(updated.dependencies, vec![f1.id]);

        // Remove dependency
        FeatureStore::remove_dependency(&conn, f2.id, f1.id).unwrap();
        let updated = FeatureStore::get_by_id(&conn, f2.id).unwrap().unwrap();
        assert!(updated.dependencies.is_empty());

        // Set dependencies
        FeatureStore::set_dependencies(&conn, f2.id, &[f1.id]).unwrap();
        let updated = FeatureStore::get_by_id(&conn, f2.id).unwrap().unwrap();
        assert_eq!(updated.dependencies, vec![f1.id]);
    }

    #[test]
    fn test_get_ready_and_blocked() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let f1 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F1".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        let f2 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F2".to_string(),
                description: "".to_string(),
                priority: Some(2),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![DependencyRef::Id(f1.id)],
            },
        )
        .unwrap();

        // F1 should be ready (no deps), F2 should be blocked (F1 not passing)
        let ready = FeatureStore::get_ready(&conn).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, f1.id);

        let blocked = FeatureStore::get_blocked(&conn).unwrap();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].id, f2.id);

        // Mark F1 as passing
        FeatureStore::mark_passing(&conn, f1.id).unwrap();

        // Now F2 should be ready
        let ready = FeatureStore::get_ready(&conn).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, f2.id);

        let blocked = FeatureStore::get_blocked(&conn).unwrap();
        assert_eq!(blocked.len(), 0);
    }

    #[test]
    fn test_get_stats() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let f1 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F1".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        let f2 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F2".to_string(),
                description: "".to_string(),
                priority: Some(2),
                category: "".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        FeatureStore::mark_passing(&conn, f1.id).unwrap();
        FeatureStore::mark_in_progress(&conn, f2.id).unwrap();

        let stats = FeatureStore::get_stats(&conn).unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.passing, 1);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.failing, 0);
    }

    #[test]
    fn test_get_graph() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.writer().lock().unwrap();

        let f1 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F1".to_string(),
                description: "".to_string(),
                priority: Some(1),
                category: "Cat1".to_string(),
                steps: vec![],
                dependencies: vec![],
            },
        )
        .unwrap();

        let f2 = FeatureStore::create(
            &conn,
            &CreateFeatureInput {
                name: "F2".to_string(),
                description: "".to_string(),
                priority: Some(2),
                category: "Cat2".to_string(),
                steps: vec![],
                dependencies: vec![DependencyRef::Id(f1.id)],
            },
        )
        .unwrap();

        let graph = FeatureStore::get_graph(&conn).unwrap();
        assert_eq!(graph.len(), 2);

        // F1 should have F2 as a dependent
        let f1_node = graph.iter().find(|n| n.id == f1.id).unwrap();
        assert_eq!(f1_node.dependents, vec![f2.id]);

        // F2 should be blocked (F1 not passing)
        let f2_node = graph.iter().find(|n| n.id == f2.id).unwrap();
        assert!(f2_node.blocked);
        assert_eq!(f2_node.dependencies, vec![f1.id]);
    }
}
