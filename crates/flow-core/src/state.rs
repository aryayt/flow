use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub current_project: Option<String>,
    pub recent_projects: Vec<String>,
    pub worktrees: Vec<WorktreeState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeState {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub created_at: u64,
}

impl State {
    /// Load state from the state directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the state file exists but cannot be read.
    pub fn load(state_dir: &Path) -> Result<Self, crate::FlowError> {
        let state_path = state_dir.join("state.toml");

        if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)?;
            Ok(toml::from_str(&content).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to the state directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be serialized or written.
    pub fn save(&self, state_dir: &Path) -> Result<(), crate::FlowError> {
        std::fs::create_dir_all(state_dir)?;
        let state_path = state_dir.join("state.toml");
        let content =
            toml::to_string_pretty(self).map_err(|e| crate::FlowError::Git(e.to_string()))?;
        std::fs::write(state_path, content)?;
        Ok(())
    }
}
