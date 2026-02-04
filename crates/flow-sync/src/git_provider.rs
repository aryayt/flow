use crate::provider::SyncProvider;
use async_trait::async_trait;
use std::path::PathBuf;

pub struct GitProvider {
    repo_path: PathBuf,
}

impl GitProvider {
    #[must_use]
    pub const fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }
}

#[async_trait]
impl SyncProvider for GitProvider {
    async fn push(&self, _data: &[u8]) -> Result<(), crate::SyncError> {
        // TODO: Implement git push for state sync
        tracing::info!("Pushing state to git repo: {:?}", self.repo_path);
        Ok(())
    }

    async fn pull(&self) -> Result<Vec<u8>, crate::SyncError> {
        // TODO: Implement git pull for state sync
        tracing::info!("Pulling state from git repo: {:?}", self.repo_path);
        Ok(Vec::new())
    }
}
