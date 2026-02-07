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
        Self {
            projects_dir: home_dir().join("Projects"),
            state_dir: state_dir().join("flow"),
            default_base_branch: "main".to_string(),
        }
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(std::env::temp_dir)
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| home_dir().join(".config"))
}

fn state_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| home_dir().join(".local").join("state"))
}

impl Config {
    /// Load configuration from `~/.config/flow/config.toml` or return defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, crate::FlowError> {
        let config_path = config_dir().join("flow").join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}
