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
pub mod gui;

// Essential security and execution modules
pub mod secure_executor;
pub mod error_handling;
pub mod input_validation;
pub mod privilege_manager;
pub mod sandboxing;
pub mod runtime;
// pub mod audit;            // Has sqlx Row issues
pub mod unified_manager;

// Advanced features - temporarily disabled for compilation
pub mod transaction;      // Fixed borrow checker issues
pub mod advanced_resolver; // Fixed version constraint issues
// pub mod secure_brain_v2;  // Depends on problematic modules

// Remote capabilities - disabled until integration complete
pub mod ssh_real;  // Fixed SSH API issues
pub mod ssh;       // Fixed SSH API issues  
pub mod docker;    // Fixed async compilation issues

// Re-export commonly used types for easier testing
pub use brain::OmniBrain;
pub use config::OmniConfig;
pub use manifest::OmniManifest;
pub use search::{SearchEngine, SearchResult};
// SecurityManager is not implemented yet
pub use database::Database;
pub use snapshot::SnapshotManager;
pub use resolver::DependencyResolver;

// Re-export essential secure components
pub use secure_executor::{SecureExecutor, ExecutionConfig, ExecutionResult};
pub use error_handling::{OmniError, RetryHandler, RecoveryManager};
pub use input_validation::InputValidator;
pub use privilege_manager::PrivilegeManager;
// pub use audit::{AuditManager, AuditEntry, SecurityEvent, SecuritySeverity, AuditConfig};
pub use unified_manager::UnifiedPackageManager;
pub use runtime::RuntimeManager;

// Re-export advanced components - temporarily disabled
pub use transaction::{TransactionManager, Transaction, TransactionType, TransactionStatus};
pub use advanced_resolver::{AdvancedDependencyResolver, ResolutionPlan, ResolutionStrategy};
// pub use secure_brain_v2::SecureOmniBrainV2;

// Remote capabilities - disabled until integration complete
pub use ssh::{SshClient, SshConfig, SshSession, AuthMethod, SshCommandResult, SystemInfo};
pub use docker::{DockerClient, DockerConfig, DockerPackageManager, ContainerInfo};