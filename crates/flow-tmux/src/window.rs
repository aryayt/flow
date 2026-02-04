use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Tmux error: {0}")]
    Tmux(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Create a new tmux window.
///
/// # Errors
///
/// Returns an error if the window cannot be created.
pub fn create(name: &str, command: Option<&str>) -> Result<(), WindowError> {
    let mut args = vec!["new-window", "-n", name];

    if let Some(cmd) = command {
        args.push(cmd);
    }

    let output = std::process::Command::new("tmux").args(&args).output()?;

    if !output.status.success() {
        return Err(WindowError::Tmux(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

/// Split the current pane horizontally.
///
/// # Errors
///
/// Returns an error if the pane cannot be split.
pub fn split_horizontal(command: Option<&str>) -> Result<(), WindowError> {
    let mut args = vec!["split-window", "-h"];

    if let Some(cmd) = command {
        args.push(cmd);
    }

    let output = std::process::Command::new("tmux").args(&args).output()?;

    if !output.status.success() {
        return Err(WindowError::Tmux(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

/// Split the current pane vertically.
///
/// # Errors
///
/// Returns an error if the pane cannot be split.
pub fn split_vertical(command: Option<&str>) -> Result<(), WindowError> {
    let mut args = vec!["split-window", "-v"];

    if let Some(cmd) = command {
        args.push(cmd);
    }

    let output = std::process::Command::new("tmux").args(&args).output()?;

    if !output.status.success() {
        return Err(WindowError::Tmux(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}
