use thiserror::Error;

/// Errors that can occur during orchestrator operations.
#[derive(Error, Debug)]
pub enum OrchestratorError {
    /// The specified agent was not found in the orchestrator's agent pool.
    #[error("agent '{0}' not found")]
    AgentNotFound(String),

    /// Attempted to spawn an agent that is already running.
    #[error("agent '{0}' already running")]
    AgentAlreadyRunning(String),

    /// No features are currently ready for assignment to agents.
    #[error("no features ready to assign")]
    NoFeaturesReady,

    /// The feature is already claimed by another agent.
    #[error("feature {0} already claimed")]
    FeatureAlreadyClaimed(i64),

    /// Maximum number of agents has been reached.
    #[error("max agents ({0}) already running")]
    MaxAgentsReached(usize),

    /// Agent process failed or crashed.
    #[error("agent process failed: {0}")]
    ProcessError(String),

    /// Database operation failed.
    #[error("database error: {0}")]
    Database(#[from] flow_core::FlowError),

    /// I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Scheduler encountered an error.
    #[error("scheduler error: {0}")]
    Scheduler(String),

    /// JSON serialization/deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// File watcher error.
    #[error("file watcher error: {0}")]
    Watcher(String),
}

/// Convenience type alias for Results using `OrchestratorError`.
pub type Result<T> = std::result::Result<T, OrchestratorError>;
