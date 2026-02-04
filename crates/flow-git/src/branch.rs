use thiserror::Error;

#[derive(Error, Debug)]
pub enum BranchError {
    #[error("Git error: {0}")]
    Git(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Get the current branch name.
///
/// # Errors
///
/// Returns an error if the branch cannot be determined.
pub fn current() -> Result<String, BranchError> {
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if !output.status.success() {
        return Err(BranchError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// List all remote branches.
///
/// # Errors
///
/// Returns an error if the branches cannot be listed.
pub fn list_remote() -> Result<Vec<String>, BranchError> {
    let output = std::process::Command::new("git")
        .args(["branch", "-r", "--format=%(refname:short)"])
        .output()?;

    if !output.status.success() {
        return Err(BranchError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(std::string::ToString::to_string)
        .collect())
}
