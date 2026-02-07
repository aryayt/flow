use serde::{Deserialize, Serialize};

/// A feature tracked in `SQLite` - the primary unit of work in Flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub id: i64,
    pub priority: i32,
    #[serde(default)]
    pub category: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub in_progress: bool,
    #[serde(default)]
    pub dependencies: Vec<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Stats summary for features.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureStats {
    pub total: usize,
    pub passing: usize,
    pub failing: usize,
    pub in_progress: usize,
    pub blocked: usize,
}

/// A node in the dependency graph visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureGraphNode {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub priority: i32,
    pub passes: bool,
    pub in_progress: bool,
    pub blocked: bool,
    pub dependencies: Vec<i64>,
    pub dependents: Vec<i64>,
}

/// Input for creating a new feature.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeatureInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<DependencyRef>,
}

/// A dependency reference - either by ID or by index within a bulk creation batch.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencyRef {
    Id(i64),
    Index { index: usize },
}
