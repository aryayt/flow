use serde::{Deserialize, Serialize};

/// Unified event type for broadcasting across TUI, SSE, and WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FlowEvent {
    /// A task JSON file was created, modified, or deleted.
    #[serde(rename = "update")]
    TaskUpdate {
        event: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
    },

    /// A feature in `SQLite` was updated.
    #[serde(rename = "feature_update")]
    FeatureUpdate {
        id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        passes: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        in_progress: Option<bool>,
    },

    /// Agent status change.
    #[serde(rename = "agent-status")]
    AgentStatus {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        task: Option<String>,
    },

    /// Progress snapshot.
    #[serde(rename = "progress")]
    Progress {
        passing: usize,
        total: usize,
        in_progress: usize,
    },

    /// Session metadata changed.
    #[serde(rename = "metadata-update")]
    MetadataUpdate,

    /// Initial connection acknowledgment.
    #[serde(rename = "connected")]
    Connected,
}
