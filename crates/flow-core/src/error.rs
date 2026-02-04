use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Tmux error: {0}")]
    Tmux(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),
}
