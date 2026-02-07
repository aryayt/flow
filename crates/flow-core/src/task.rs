use serde::{Deserialize, Serialize};

/// A task from Claude Code's JSON file format (~/.claude/tasks/{session}/{id}.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub subject: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "activeForm")]
    pub active_form: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default, rename = "blockedBy")]
    pub blocked_by: Vec<String>,
}

/// Session list item for the web UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionListItem {
    pub id: String,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub project: Option<String>,
    pub description: Option<String>,
    pub git_branch: Option<String>,
    pub task_count: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub pending: usize,
    pub created_at: Option<String>,
    pub modified_at: String,
}

/// A task with its session context.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWithSession {
    #[serde(flatten)]
    pub task: Task,
    pub session_id: String,
    pub session_name: Option<String>,
    pub project: Option<String>,
}

/// Session metadata extracted from JSONL files.
#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub custom_title: Option<String>,
    pub slug: Option<String>,
    pub project_path: Option<String>,
    pub description: Option<String>,
    pub git_branch: Option<String>,
    pub created: Option<String>,
}

impl SessionMeta {
    /// Get display name: customTitle > slug > None.
    pub fn display_name(&self) -> Option<String> {
        self.custom_title.clone().or_else(|| self.slug.clone())
    }
}
