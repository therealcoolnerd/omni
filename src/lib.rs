// Omni Universal Linux Package Manager Library
// This file exposes the public API for testing and benchmarking

pub mod brain;
pub mod config;
pub mod manifest;
pub mod search;
pub mod security;
pub mod database;
pub mod snapshot;
pub mod resolver;
pub mod distro;
pub mod history;
pub mod updater;
pub mod interactive;
pub mod logging;
pub mod boxes;

// Re-export commonly used types for easier testing
pub use brain::OmniBrain;
pub use config::OmniConfig;
pub use manifest::OmniManifest;
pub use search::{SearchEngine, SearchResult};
pub use security::SecurityManager;
pub use database::Database;
pub use snapshot::SnapshotManager;
pub use resolver::DependencyResolver;