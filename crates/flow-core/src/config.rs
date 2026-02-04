use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub projects_dir: PathBuf,
    pub state_dir: PathBuf,
    pub default_base_branch: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir();
        Self {
            projects_dir: home.join("Projects"),
            state_dir: home.join(".local/state/flow"),
            default_base_branch: "main".to_string(),
        }
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME").map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

impl Config {
    /// Load configuration from `~/.config/flow/config.toml` or return defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, crate::FlowError> {
        let config_path = home_dir().join(".config/flow/config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}
