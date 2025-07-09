use anyhow::Result;
use std::future::Future;

pub struct RuntimeManager;

impl RuntimeManager {
    pub fn new() -> Self {
        Self
    }

    pub fn check_system_health(&self) -> Result<()> {
        Ok(())
    }

    pub fn block_on<F, T>(future: F) -> T
    where
        F: Future<Output = T>,
    {
        // Use the current tokio runtime to block on the future
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(future)
        })
    }
}
