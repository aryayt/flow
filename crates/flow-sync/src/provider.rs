use async_trait::async_trait;

#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// Push state data to the remote provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the push fails.
    async fn push(&self, data: &[u8]) -> Result<(), crate::SyncError>;

    /// Pull state data from the remote provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the pull fails.
    async fn pull(&self) -> Result<Vec<u8>, crate::SyncError>;
}
