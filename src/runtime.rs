use std::sync::OnceLock;
use tokio::runtime::{Handle, Runtime};
use anyhow::Result;

/// Global runtime manager for the application
pub struct RuntimeManager {
    handle: Handle,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

impl RuntimeManager {
    /// Get or create the global runtime
    pub fn global() -> &'static Runtime {
        RUNTIME.get_or_init(|| {
            Runtime::new().expect("Failed to create tokio runtime")
        })
    }
    
    /// Get the current runtime handle
    pub fn handle() -> Result<Handle> {
        match Handle::try_current() {
            Ok(handle) => Ok(handle),
            Err(_) => Ok(Self::global().handle().clone()),
        }
    }
    
    /// Block on a future using the global runtime
    pub fn block_on<F>(future: F) -> F::Output
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match Handle::try_current() {
            Ok(handle) => {
                // We're already in an async context, need to spawn
                std::thread::spawn(move || {
                    Self::global().block_on(future)
                }).join().expect("Failed to join thread")
            }
            Err(_) => {
                // We're in a sync context, can block directly
                Self::global().block_on(future)
            }
        }
    }
    
    /// Spawn a task on the global runtime
    pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Self::global().spawn(future)
    }
}