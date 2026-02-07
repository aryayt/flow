use crate::theme::Theme;
use std::path::PathBuf;

/// Configuration for the agent monitoring system.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Port for the web server (default: 3456).
    pub port: u16,

    /// Root directory for Claude/agent data (e.g., ~/.claude).
    pub claude_dir: PathBuf,

    /// Data directory for application state.
    pub data_dir: PathBuf,

    /// UI theme.
    pub theme: Theme,

    /// Maximum number of concurrent agent operations.
    pub max_concurrency: usize,

    /// Testing ratio for validation (0.0-1.0).
    pub testing_ratio: f64,

    /// Batch size for bulk operations.
    pub batch_size: usize,

    /// Enable yolo mode (skip confirmations).
    pub yolo_mode: bool,

    /// Optional custom public directory for static assets.
    pub public_dir: Option<PathBuf>,

    /// Open browser on server start.
    pub open_browser: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let home = dirs::home_dir().expect("Failed to get home directory");
        let claude_dir = home.join(".claude");
        let data_dir = claude_dir.join("data");

        Self {
            port: 3456,
            claude_dir,
            data_dir,
            theme: Theme::default(),
            max_concurrency: 4,
            testing_ratio: 0.3,
            batch_size: 10,
            yolo_mode: false,
            public_dir: None,
            open_browser: true,
        }
    }
}

impl AgentConfig {
    /// Create a new `AgentConfig` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the tasks directory path (~/.claude/tasks).
    #[must_use]
    pub fn tasks_dir(&self) -> PathBuf {
        self.claude_dir.join("tasks")
    }

    /// Get the projects directory path (~/.claude/projects).
    #[must_use]
    pub fn projects_dir(&self) -> PathBuf {
        self.claude_dir.join("projects")
    }

    /// Get the teams directory path (~/.claude/teams).
    #[must_use]
    pub fn teams_dir(&self) -> PathBuf {
        self.claude_dir.join("teams")
    }

    /// Get the features database path for a specific project.
    #[must_use]
    pub fn features_db_path(&self, project: &str) -> PathBuf {
        self.data_dir.join(format!("{project}.db"))
    }

    /// Set a custom port.
    #[must_use]
    pub const fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set a custom claude directory.
    #[must_use]
    pub fn with_claude_dir(mut self, dir: PathBuf) -> Self {
        self.claude_dir = dir;
        self
    }

    /// Set a custom theme.
    #[must_use]
    pub const fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set yolo mode.
    #[must_use]
    pub const fn with_yolo_mode(mut self, enabled: bool) -> Self {
        self.yolo_mode = enabled;
        self
    }

    /// Set custom public directory.
    #[must_use]
    pub fn with_public_dir(mut self, dir: PathBuf) -> Self {
        self.public_dir = Some(dir);
        self
    }

    /// Set whether to open browser on start.
    #[must_use]
    pub const fn with_open_browser(mut self, open: bool) -> Self {
        self.open_browser = open;
        self
    }
}
