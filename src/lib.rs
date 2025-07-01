// Omni Universal Linux Package Manager Library
// This file exposes the public API for testing and benchmarking

pub mod boxes;
pub mod brain;
pub mod config;
pub mod database;
pub mod distro;
#[cfg(feature = "gui")]
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
pub mod types;
pub mod updater;

// Essential security and execution modules
pub mod error_handling;
pub mod input_validation;
pub mod privilege_manager;
pub mod runtime;
pub mod sandboxing;
pub mod secure_executor;
pub mod audit;
pub mod unified_manager;

// Advanced features - refactored for stability
pub mod advanced_resolver_v2;
pub mod transaction_v2;
pub mod secure_brain_v2;

// Remote capabilities - feature gated
#[cfg(feature = "ssh")]
pub mod ssh;
#[cfg(feature = "ssh")]
pub mod ssh_real;

// Container support - optional
pub mod docker;

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
pub use error_handling::{
    OmniError, RecoveryManager, RetryHandler, ErrorContext, ErrorCategory, ErrorSeverity,
    RecoveryStrategy, ErrorMonitor, ErrorMetrics, RecoveryMetrics, get_error_monitor, record_error
};
pub use input_validation::InputValidator;
pub use privilege_manager::PrivilegeManager;
pub use secure_executor::{ExecutionConfig, ExecutionResult, SecureExecutor};
// pub use audit::{AuditManager, AuditEntry, SecurityEvent, SecuritySeverity, AuditConfig};
pub use runtime::RuntimeManager;
pub use unified_manager::UnifiedPackageManager;

// Re-export advanced components - refactored versions
pub use advanced_resolver_v2::{AdvancedDependencyResolver, ResolutionPlan, ResolutionStrategy, PackageAction, ActionType};
pub use transaction_v2::{Transaction, TransactionManager, TransactionStatus, TransactionType, Operation, OperationType, OperationStatus};
// pub use secure_brain_v2::SecureOmniBrainV2;

// Re-export common types
pub use types::InstalledPackage;

// Re-export enhanced package managers
pub use boxes::apt::AptManager;
pub use boxes::dnf::DnfBox;
pub use boxes::winget::WingetBox;
pub use boxes::brew::BrewBox;
pub use boxes::snap::SnapBox;

// Remote capabilities - disabled until integration complete
// pub use docker::{ContainerInfo, DockerClient, DockerConfig, DockerPackageManager};
// pub use ssh::{AuthMethod, SshClient, SshCommandResult, SshConfig, SshSession, SystemInfo};
