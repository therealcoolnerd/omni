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
pub mod rate_limiter;

// New secure and improved modules
pub mod secure_executor;
pub mod transaction;
pub mod advanced_resolver;
pub mod error_handling;
pub mod input_validation;
pub mod privilege_manager;
pub mod sandboxing;
pub mod secure_brain;

// Re-export commonly used types for easier testing
pub use brain::OmniBrain;
pub use config::OmniConfig;
pub use manifest::OmniManifest;
pub use search::{SearchEngine, SearchResult};
pub use security::SecurityManager;
pub use database::Database;
pub use snapshot::SnapshotManager;
pub use resolver::DependencyResolver;

// Re-export new secure components
pub use secure_executor::{SecureExecutor, ExecutionConfig, ExecutionResult};
pub use transaction::{TransactionManager, Transaction, TransactionType, TransactionResult};
pub use advanced_resolver::{AdvancedDependencyResolver, ResolutionPlan, ResolutionStrategy};
pub use error_handling::{OmniError, RetryHandler, RecoveryManager};
pub use input_validation::InputValidator;
pub use privilege_manager::PrivilegeManager;
pub use secure_brain::{SecureOmniBrain, SystemStatus};