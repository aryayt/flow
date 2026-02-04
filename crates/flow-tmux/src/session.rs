use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Tmux error: {0}")]
    Tmux(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Open a tmux session for a worktree path.
///
/// # Errors
///
/// Returns an error if the session cannot be created or switched to.
pub fn open_worktree(path: &Path) -> Result<(), SessionError> {
    let session_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("worktree");

    // Check if session exists
    let check = std::process::Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .output()?;

    if !check.status.success() {
        // Create new session
        let output = std::process::Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                session_name,
                "-c",
                path.to_str().unwrap_or("."),
            ])
            .output()?;

        if !output.status.success() {
            return Err(SessionError::Tmux(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
    }

    // Switch to session
    std::process::Command::new("tmux")
        .args(["switch-client", "-t", session_name])
        .output()?;

    tracing::info!("Opened tmux session: {}", session_name);
    Ok(())
}

/// Switch to a project path in tmux.
///
/// # Errors
///
/// Returns an error if the session cannot be switched to.
pub fn switch_to(path: &Path) -> Result<(), SessionError> {
    open_worktree(path)
}

/// List all tmux sessions.
///
/// # Errors
///
/// Returns an error if the sessions cannot be listed.
pub fn list_sessions() -> Result<Vec<String>, SessionError> {
    let output = std::process::Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new()); // No sessions or tmux not running
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(std::string::ToString::to_string)
        .collect())
}
