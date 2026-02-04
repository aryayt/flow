use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorktreeError {
    #[error("Git error: {0}")]
    Git(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Core error: {0}")]
    Core(#[from] flow_core::FlowError),
}

#[derive(Debug, Clone)]
pub struct Worktree {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
}

/// Create a new git worktree.
///
/// # Errors
///
/// Returns an error if the worktree cannot be created.
pub fn create(name: &str, base: &str) -> Result<PathBuf, WorktreeError> {
    let config = flow_core::Config::load()?;
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    let worktree_path = config
        .projects_dir
        .join(format!("{project_name}-worktrees/{name}"));

    let output = std::process::Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            name,
            worktree_path.to_str().unwrap_or("."),
            base,
        ])
        .output()?;

    if !output.status.success() {
        return Err(WorktreeError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    tracing::info!("Created worktree at {:?}", worktree_path);
    Ok(worktree_path)
}

/// List all git worktrees in the current repository.
///
/// # Errors
///
/// Returns an error if the worktrees cannot be listed.
pub fn list() -> Result<Vec<Worktree>, WorktreeError> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Err(WorktreeError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let mut worktrees = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_string());
        } else if line.is_empty() {
            if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                worktrees.push(Worktree { name, path, branch });
            }
        }
    }

    Ok(worktrees)
}

/// Remove a git worktree.
///
/// # Errors
///
/// Returns an error if the worktree cannot be removed.
pub fn remove(name: &str) -> Result<(), WorktreeError> {
    let output = std::process::Command::new("git")
        .args(["worktree", "remove", name])
        .output()?;

    if !output.status.success() {
        return Err(WorktreeError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    tracing::info!("Removed worktree: {}", name);
    Ok(())
}
