//! Multi-machine state synchronization for Agent Flow.
//!
//! Synchronizes workspace state across machines using pluggable providers
//! (git-based sync, file-based sync). Currently a work-in-progress.

pub mod git_provider;
pub mod provider;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Sync error: {0}")]
    Sync(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Sync state across machines.
///
/// # Errors
///
/// Returns an error if the state cannot be synced.
pub fn sync_state() -> anyhow::Result<()> {
    let config = flow_core::Config::load()?;
    let state = flow_core::state::State::load(&config.state_dir)?;

    // TODO: Implement actual sync via git or other provider
    tracing::info!("Syncing state: {:?}", state);

    Ok(())
}
