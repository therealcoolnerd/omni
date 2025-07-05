// Omni Universal Linux Package Manager Library
// This file exposes the public API for testing and benchmarking

pub mod boxes;
pub mod brain;
pub mod config;
pub mod database;
pub mod distro;
#[cfg(feature = "gui")]
pub mod gui;
pub mod hardware;
pub mod history;
pub mod interactive;
pub mod logging;
pub mod manifest;
pub mod resolver;
pub mod search;
pub mod security;
pub mod snapshot;
pub mod types;
pub mod updater;

// Essential modules
pub mod advanced_resolver;
pub mod audit;
pub mod error_handling;
pub mod input_validation;
pub mod privilege_manager;
pub mod runtime;
pub mod sandboxing;
pub mod secure_brain;
pub mod secure_executor;
pub mod transaction;
pub mod unified_manager;

// Remote capabilities - feature gated
#[cfg(feature = "ssh")]
pub mod ssh;

// Container support - optional
pub mod docker;

// Re-export commonly used types for easier testing
pub use brain::OmniBrain;
pub use config::OmniConfig;
pub use hardware::{detect_and_suggest_drivers, HardwareDetector, HardwareInfo};
pub use manifest::OmniManifest;
pub use search::{SearchEngine, SearchResult};
// SecurityManager is not implemented yet
pub use database::Database;
pub use resolver::DependencyResolver;
pub use snapshot::SnapshotManager;

// Re-export essential components
pub use error_handling::OmniError;
pub use input_validation::InputValidator;
pub use unified_manager::UnifiedPackageManager;

// Re-export advanced components
pub use advanced_resolver::AdvancedDependencyResolver;
pub use transaction::TransactionManager;

// Re-export common types
pub use types::InstalledPackage;

// Re-export package managers
pub use boxes::apt::AptManager;
