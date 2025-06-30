// Omni Universal Linux Package Manager Library
// This file exposes the public API for testing and benchmarking

pub mod boxes;
pub mod brain;
pub mod config;
pub mod database;
pub mod distro;
pub mod gui;
pub mod history;
pub mod interactive;
pub mod logging;
pub mod manifest;
pub mod rate_limiter;
pub mod resolver;
pub mod search;
pub mod security;
pub mod snapshot;
pub mod updater;

// Essential security and execution modules
pub mod error_handling;
pub mod input_validation;
pub mod privilege_manager;
pub mod runtime;
pub mod sandboxing;
pub mod secure_executor;
// pub mod audit;            // Has sqlx Row issues
pub mod unified_manager;

// Advanced features - temporarily disabled for compilation
pub mod advanced_resolver;
pub mod transaction; // Fixed borrow checker issues // Fixed version constraint issues
                                                    // pub mod secure_brain_v2;  // Depends on problematic modules

// Remote capabilities - disabled until integration complete
pub mod docker;
pub mod ssh; // Fixed SSH API issues
pub mod ssh_real; // Fixed SSH API issues // Fixed async compilation issues

// Re-export commonly used types for easier testing
pub use brain::OmniBrain;
pub use config::OmniConfig;
pub use manifest::OmniManifest;
pub use search::{SearchEngine, SearchResult};
// SecurityManager is not implemented yet
pub use database::Database;
pub use resolver::DependencyResolver;
pub use snapshot::SnapshotManager;

// Re-export essential secure components
pub use error_handling::{OmniError, RecoveryManager, RetryHandler};
pub use input_validation::InputValidator;
pub use privilege_manager::PrivilegeManager;
pub use secure_executor::{ExecutionConfig, ExecutionResult, SecureExecutor};
// pub use audit::{AuditManager, AuditEntry, SecurityEvent, SecuritySeverity, AuditConfig};
pub use runtime::RuntimeManager;
pub use unified_manager::UnifiedPackageManager;

// Re-export advanced components - temporarily disabled
pub use advanced_resolver::{AdvancedDependencyResolver, ResolutionPlan, ResolutionStrategy};
pub use transaction::{Transaction, TransactionManager, TransactionStatus, TransactionType};
// pub use secure_brain_v2::SecureOmniBrainV2;

// Remote capabilities - disabled until integration complete
pub use docker::{ContainerInfo, DockerClient, DockerConfig, DockerPackageManager};
pub use ssh::{AuthMethod, SshClient, SshCommandResult, SshConfig, SshSession, SystemInfo};
