use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Comprehensive error types for Omni
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OmniError {
    #[error("Package not found: {package}")]
    PackageNotFound { package: String },

    #[error("Box type not supported: {box_type}")]
    UnsupportedBoxType { box_type: String },

    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        url: Option<String>,
    },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Installation failed: {package} via {box_type}: {reason}")]
    InstallationFailed {
        package: String,
        box_type: String,
        reason: String,
    },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Security violation: {message}")]
    SecurityViolation { message: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Timeout occurred: {operation} after {duration:?}")]
    TimeoutError {
        operation: String,
        duration: Duration,
    },

    #[error("Validation error: {field}: {message}")]
    ValidationError { field: String, message: String },

    #[error("Recovery failed: {message}")]
    RecoveryFailed { message: String },

    #[error("Unknown error: {message}")]
    Unknown { message: String },

    #[error("Dependency resolution failed: {message}")]
    DependencyResolutionFailed { message: String },

    #[error("Transaction failed: {transaction_id}: {reason}")]
    TransactionFailed {
        transaction_id: String,
        reason: String,
    },

    #[error("Snapshot operation failed: {operation}: {reason}")]
    SnapshotFailed { operation: String, reason: String },

    #[error("Cache operation failed: {operation}: {reason}")]
    CacheFailed { operation: String, reason: String },
}

impl From<anyhow::Error> for OmniError {
    fn from(err: anyhow::Error) -> Self {
        OmniError::Unknown {
            message: err.to_string(),
        }
    }
}

impl OmniError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            OmniError::NetworkError { .. } => true,
            OmniError::DatabaseError { .. } => true,
            OmniError::TimeoutError { .. } => true,
            OmniError::ResourceExhausted { .. } => true,
            OmniError::PackageNotFound { .. } => false,
            OmniError::UnsupportedBoxType { .. } => false,
            OmniError::PermissionDenied { .. } => false,
            OmniError::SecurityViolation { .. } => false,
            OmniError::ValidationError { .. } => false,
            OmniError::ConfigurationError { .. } => false,
            OmniError::InstallationFailed { .. } => true, // Might be transient
            OmniError::RecoveryFailed { .. } => false,
            OmniError::Unknown { .. } => false,
            OmniError::DependencyResolutionFailed { .. } => true,
            OmniError::TransactionFailed { .. } => true,
            OmniError::SnapshotFailed { .. } => true,
            OmniError::CacheFailed { .. } => true,
        }
    }

    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            OmniError::SecurityViolation { .. } => ErrorSeverity::Critical,
            OmniError::PermissionDenied { .. } => ErrorSeverity::High,
            OmniError::InstallationFailed { .. } => ErrorSeverity::High,
            OmniError::ConfigurationError { .. } => ErrorSeverity::Medium,
            OmniError::ValidationError { .. } => ErrorSeverity::Medium,
            OmniError::NetworkError { .. } => ErrorSeverity::Medium,
            OmniError::DatabaseError { .. } => ErrorSeverity::Medium,
            OmniError::PackageNotFound { .. } => ErrorSeverity::Low,
            OmniError::UnsupportedBoxType { .. } => ErrorSeverity::Low,
            OmniError::TimeoutError { .. } => ErrorSeverity::Low,
            OmniError::ResourceExhausted { .. } => ErrorSeverity::Medium,
            OmniError::RecoveryFailed { .. } => ErrorSeverity::High,
            OmniError::Unknown { .. } => ErrorSeverity::Medium,
            OmniError::DependencyResolutionFailed { .. } => ErrorSeverity::High,
            OmniError::TransactionFailed { .. } => ErrorSeverity::High,
            OmniError::SnapshotFailed { .. } => ErrorSeverity::Medium,
            OmniError::CacheFailed { .. } => ErrorSeverity::Low,
        }
    }

    /// Get suggested recovery actions
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            OmniError::PackageNotFound { package } => vec![
                format!("Check package name spelling: {}", package),
                "Try searching for similar packages".to_string(),
                "Verify package is available in enabled repositories".to_string(),
            ],
            OmniError::UnsupportedBoxType { box_type } => vec![
                format!("Install {} package manager", box_type),
                "Use --box-type to specify a different package manager".to_string(),
                "Check supported package managers with 'omni config show'".to_string(),
            ],
            OmniError::NetworkError { .. } => vec![
                "Check internet connection".to_string(),
                "Verify repository URLs are accessible".to_string(),
                "Try again in a few minutes".to_string(),
                "Use --offline mode if available".to_string(),
            ],
            OmniError::PermissionDenied { .. } => vec![
                "Run with sudo if required".to_string(),
                "Check file/directory permissions".to_string(),
                "Verify user has necessary privileges".to_string(),
            ],
            OmniError::InstallationFailed { .. } => vec![
                "Check system requirements".to_string(),
                "Verify sufficient disk space".to_string(),
                "Try updating package repositories".to_string(),
                "Check for conflicting packages".to_string(),
            ],
            OmniError::DatabaseError { .. } => vec![
                "Check database file permissions".to_string(),
                "Verify sufficient disk space".to_string(),
                "Try clearing corrupted cache".to_string(),
            ],
            OmniError::ConfigurationError { .. } => vec![
                "Check configuration file syntax".to_string(),
                "Reset to default configuration".to_string(),
                "Verify all required fields are present".to_string(),
            ],
            OmniError::SecurityViolation { .. } => vec![
                "Review security settings".to_string(),
                "Verify package signatures".to_string(),
                "Do not proceed if unsure about safety".to_string(),
            ],
            OmniError::ResourceExhausted { resource } => vec![
                format!("Free up {}", resource),
                "Close unnecessary applications".to_string(),
                "Check system resource usage".to_string(),
            ],
            OmniError::TimeoutError { .. } => vec![
                "Increase timeout setting".to_string(),
                "Check network connectivity".to_string(),
                "Try again during off-peak hours".to_string(),
            ],
            OmniError::ValidationError { .. } => vec![
                "Check input format and requirements".to_string(),
                "Refer to documentation for valid values".to_string(),
            ],
            OmniError::DependencyResolutionFailed { .. } => vec![
                "Check for circular dependencies".to_string(),
                "Try resolving dependencies manually".to_string(),
                "Update package repositories".to_string(),
                "Use --force-deps flag if appropriate".to_string(),
            ],
            OmniError::TransactionFailed { .. } => vec![
                "Review transaction operations".to_string(),
                "Check system resources".to_string(),
                "Retry with smaller transaction batches".to_string(),
                "Verify package availability".to_string(),
            ],
            OmniError::SnapshotFailed { .. } => vec![
                "Check filesystem permissions".to_string(),
                "Verify sufficient disk space".to_string(),
                "Try creating snapshot with different name".to_string(),
                "Clean up corrupted snapshot data".to_string(),
            ],
            OmniError::CacheFailed { .. } => vec![
                "Clear package cache".to_string(),
                "Update repository metadata".to_string(),
                "Check network connectivity".to_string(),
                "Retry with --no-cache flag".to_string(),
            ],
            _ => vec!["Check logs for more details".to_string()],
        }
    }

    /// Get error category for grouping and analytics
    pub fn category(&self) -> ErrorCategory {
        match self {
            OmniError::PackageNotFound { .. } => ErrorCategory::Package,
            OmniError::UnsupportedBoxType { .. } => ErrorCategory::System,
            OmniError::NetworkError { .. } => ErrorCategory::Network,
            OmniError::DatabaseError { .. } => ErrorCategory::Storage,
            OmniError::PermissionDenied { .. } => ErrorCategory::Security,
            OmniError::InstallationFailed { .. } => ErrorCategory::Package,
            OmniError::ConfigurationError { .. } => ErrorCategory::Configuration,
            OmniError::SecurityViolation { .. } => ErrorCategory::Security,
            OmniError::ResourceExhausted { .. } => ErrorCategory::System,
            OmniError::TimeoutError { .. } => ErrorCategory::Network,
            OmniError::ValidationError { .. } => ErrorCategory::Validation,
            OmniError::RecoveryFailed { .. } => ErrorCategory::Recovery,
            OmniError::DependencyResolutionFailed { .. } => ErrorCategory::Dependencies,
            OmniError::TransactionFailed { .. } => ErrorCategory::Transaction,
            OmniError::SnapshotFailed { .. } => ErrorCategory::Storage,
            OmniError::CacheFailed { .. } => ErrorCategory::Storage,
            OmniError::Unknown { .. } => ErrorCategory::Unknown,
        }
    }

    /// Get user-friendly error code for documentation and support
    pub fn error_code(&self) -> &'static str {
        match self {
            OmniError::PackageNotFound { .. } => "OMNI_PKG_001",
            OmniError::UnsupportedBoxType { .. } => "OMNI_SYS_001",
            OmniError::NetworkError { .. } => "OMNI_NET_001",
            OmniError::DatabaseError { .. } => "OMNI_DB_001",
            OmniError::PermissionDenied { .. } => "OMNI_SEC_001",
            OmniError::InstallationFailed { .. } => "OMNI_PKG_002",
            OmniError::ConfigurationError { .. } => "OMNI_CFG_001",
            OmniError::SecurityViolation { .. } => "OMNI_SEC_002",
            OmniError::ResourceExhausted { .. } => "OMNI_SYS_002",
            OmniError::TimeoutError { .. } => "OMNI_NET_002",
            OmniError::ValidationError { .. } => "OMNI_VAL_001",
            OmniError::RecoveryFailed { .. } => "OMNI_REC_001",
            OmniError::DependencyResolutionFailed { .. } => "OMNI_DEP_001",
            OmniError::TransactionFailed { .. } => "OMNI_TXN_001",
            OmniError::SnapshotFailed { .. } => "OMNI_SNP_001",
            OmniError::CacheFailed { .. } => "OMNI_CHE_001",
            OmniError::Unknown { .. } => "OMNI_UNK_001",
        }
    }
}

/// Error categories for grouping and analytics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    Package,
    System,
    Network,
    Storage,
    Security,
    Configuration,
    Validation,
    Recovery,
    Dependencies,
    Transaction,
    Unknown,
}

/// Error context for tracking error details and recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: Uuid,
    pub error: OmniError,
    pub timestamp: u64,
    pub operation: String,
    pub context_data: HashMap<String, String>,
    pub retry_count: usize,
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub attempt_id: Uuid,
    pub strategy: RecoveryStrategy,
    pub timestamp: u64,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    Retry,
    Fallback,
    UserIntervention,
    SystemRepair,
    ConfigurationReset,
    CacheClear,
    PermissionEscalation,
    AlternativeSource,
}

impl ErrorContext {
    pub fn new(error: OmniError, operation: String) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            error,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            operation,
            context_data: HashMap::new(),
            retry_count: 0,
            recovery_attempts: Vec::new(),
            resolved: false,
        }
    }

    pub fn add_context(&mut self, key: String, value: String) {
        self.context_data.insert(key, value);
    }

    pub fn add_recovery_attempt(
        &mut self,
        strategy: RecoveryStrategy,
        success: bool,
        details: String,
    ) {
        let attempt = RecoveryAttempt {
            attempt_id: Uuid::new_v4(),
            strategy,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success,
            details,
        };
        self.recovery_attempts.push(attempt);

        if success {
            self.resolved = true;
        }
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Package => write!(f, "Package"),
            ErrorCategory::System => write!(f, "System"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Storage => write!(f, "Storage"),
            ErrorCategory::Security => write!(f, "Security"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Recovery => write!(f, "Recovery"),
            ErrorCategory::Dependencies => write!(f, "Dependencies"),
            ErrorCategory::Transaction => write!(f, "Transaction"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new_network() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(15),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    pub fn new_database() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    pub fn new_critical() -> Self {
        Self {
            max_attempts: 1,
            base_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            backoff_multiplier: 1.0,
            jitter: false,
        }
    }
}

/// Enhanced retry mechanism with error context tracking
#[derive(Debug, Clone)]
pub struct RetryHandler {
    config: RetryConfig,
    error_contexts: std::sync::Arc<std::sync::Mutex<HashMap<String, ErrorContext>>>,
}

impl RetryHandler {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            error_contexts: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Execute a function with enhanced retry logic and error context tracking
    pub async fn execute_with_context<F, Fut, T>(
        &self,
        operation_name: &str,
        mut operation: F,
    ) -> Result<T, OmniError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, OmniError>>,
    {
        let mut error_context = ErrorContext::new(
            OmniError::Unknown {
                message: "Placeholder".to_string(),
            },
            operation_name.to_string(),
        );

        let mut last_error = None;

        for attempt in 1..=self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            "Operation '{}' succeeded on attempt {}",
                            operation_name, attempt
                        );
                        error_context.add_recovery_attempt(
                            RecoveryStrategy::Retry,
                            true,
                            format!("Succeeded on attempt {}", attempt),
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    warn!(
                        "Operation '{}' failed on attempt {}: {}",
                        operation_name, attempt, error
                    );
                    error_context.error = error.clone();
                    error_context.increment_retry();
                    last_error = Some(error.clone());

                    if attempt < self.config.max_attempts && error.is_retryable() {
                        let delay = self.calculate_delay(attempt);
                        info!("Retrying '{}' in {:?}", operation_name, delay);

                        error_context.add_recovery_attempt(
                            RecoveryStrategy::Retry,
                            false,
                            format!("Attempt {} failed, retrying in {:?}", attempt, delay),
                        );

                        sleep(delay).await;
                    } else if !error.is_retryable() {
                        error!(
                            "Operation '{}' failed with non-retryable error: {}",
                            operation_name, error
                        );
                        break;
                    }
                }
            }
        }

        // Store error context for analysis
        if let Ok(mut contexts) = self.error_contexts.lock() {
            contexts.insert(error_context.error_id.to_string(), error_context);
        }

        error!(
            "Operation '{}' failed after {} attempts",
            operation_name, self.config.max_attempts
        );

        Err(last_error.unwrap_or_else(|| OmniError::Unknown {
            message: "Unknown error occurred during retry attempts".to_string(),
        }))
    }

    /// Simplified execute method for backward compatibility
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + Clone + From<anyhow::Error>,
    {
        let mut last_error = None;

        for attempt in 1..=self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    warn!("Operation failed on attempt {}: {:?}", attempt, error);
                    last_error = Some(error.clone());

                    if attempt < self.config.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        info!("Retrying in {:?}", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        error!(
            "Operation failed after {} attempts",
            self.config.max_attempts
        );
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Unknown error occurred during retry attempts").into()
        }))
    }

    fn calculate_delay(&self, attempt: usize) -> Duration {
        let exponential_delay = self.config.base_delay.as_millis() as f64
            * self.config.backoff_multiplier.powi(attempt as i32 - 1);

        let mut delay = Duration::from_millis(exponential_delay as u64);

        // Cap at max delay
        if delay > self.config.max_delay {
            delay = self.config.max_delay;
        }

        // Add jitter to avoid thundering herd
        if self.config.jitter {
            use rand::Rng;
            let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
            delay = Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64);
        }

        delay
    }

    /// Get error contexts for analysis
    pub fn get_error_contexts(&self) -> HashMap<String, ErrorContext> {
        match self.error_contexts.lock() {
            Ok(contexts) => contexts.clone(),
            Err(_) => {
                warn!("Error contexts mutex poisoned, returning empty map");
                HashMap::new()
            }
        }
    }

    /// Clear old error contexts (cleanup)
    pub fn cleanup_old_contexts(&self, max_age_seconds: u64) {
        if let Ok(mut contexts) = self.error_contexts.lock() {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            contexts.retain(|_, ctx| current_time - ctx.timestamp < max_age_seconds);
        }
    }
}

/// Circuit breaker pattern for handling cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    failure_count: std::sync::Arc<std::sync::Mutex<usize>>,
    last_failure_time: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    state: std::sync::Arc<std::sync::Mutex<CircuitState>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,   // Normal operation
    Open,     // Rejecting requests
    HalfOpen, // Testing if service is recovered
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            last_failure_time: std::sync::Arc::new(std::sync::Mutex::new(None)),
            state: std::sync::Arc::new(std::sync::Mutex::new(CircuitState::Closed)),
        }
    }

    /// Execute operation through circuit breaker
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + From<anyhow::Error>,
    {
        // Check if circuit is open
        if self.is_open() {
            return Err(self.create_circuit_open_error());
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }

    fn is_open(&self) -> bool {
        let state = match self.state.lock() {
            Ok(state) => *state,
            Err(_) => {
                warn!("Circuit breaker state mutex poisoned, assuming open");
                return true;
            }
        };

        match state {
            CircuitState::Closed => false,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Ok(last_failure_time) = self.last_failure_time.lock() {
                    if let Some(last_failure) = *last_failure_time {
                        if last_failure.elapsed() >= self.recovery_timeout {
                            if let Ok(mut state_guard) = self.state.lock() {
                                *state_guard = CircuitState::HalfOpen;
                                info!("Circuit breaker transitioning to half-open");
                                return false;
                            }
                        }
                    }
                }
                true
            }
            CircuitState::HalfOpen => false,
        }
    }

    fn on_success(&self) {
        if let Ok(mut failure_count) = self.failure_count.lock() {
            *failure_count = 0;
        }

        if let Ok(mut state) = self.state.lock() {
            match *state {
                CircuitState::HalfOpen => {
                    *state = CircuitState::Closed;
                    info!("Circuit breaker closed after successful recovery");
                }
                _ => {}
            }
        }
    }

    fn on_failure(&self) {
        if let Ok(mut failure_count) = self.failure_count.lock() {
            *failure_count += 1;

            if let Ok(mut last_failure_time) = self.last_failure_time.lock() {
                *last_failure_time = Some(std::time::Instant::now());
            }

            if *failure_count >= self.failure_threshold {
                if let Ok(mut state) = self.state.lock() {
                    if *state != CircuitState::Open {
                        *state = CircuitState::Open;
                        warn!("Circuit breaker opened due to {} failures", *failure_count);
                    }
                }
            }
        }
    }

    fn create_circuit_open_error<E>(&self) -> E
    where
        E: From<anyhow::Error>,
    {
        E::from(anyhow!("Circuit breaker is open"))
    }
}

/// Enhanced recovery manager with automatic error recovery workflows
#[derive(Debug)]
pub struct RecoveryManager {
    retry_handler: RetryHandler,
    circuit_breaker: CircuitBreaker,
    recovery_strategies: HashMap<ErrorCategory, Vec<RecoveryStrategy>>,
    auto_recovery_enabled: bool,
    metrics: RecoveryMetrics,
}

/// Recovery metrics for monitoring and analytics
#[derive(Debug, Clone, Default)]
pub struct RecoveryMetrics {
    pub total_errors: u64,
    pub total_recoveries: u64,
    pub recovery_success_rate: f64,
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    pub recoveries_by_strategy: HashMap<RecoveryStrategy, u64>,
}

impl RecoveryManager {
    pub fn new() -> Self {
        let mut recovery_strategies = HashMap::new();

        // Define default recovery strategies for each error category
        recovery_strategies.insert(
            ErrorCategory::Network,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::AlternativeSource],
        );

        recovery_strategies.insert(
            ErrorCategory::Storage,
            vec![
                RecoveryStrategy::CacheClear,
                RecoveryStrategy::SystemRepair,
                RecoveryStrategy::Retry,
            ],
        );

        recovery_strategies.insert(
            ErrorCategory::Package,
            vec![
                RecoveryStrategy::AlternativeSource,
                RecoveryStrategy::Retry,
                RecoveryStrategy::Fallback,
            ],
        );

        recovery_strategies.insert(
            ErrorCategory::Security,
            vec![
                RecoveryStrategy::PermissionEscalation,
                RecoveryStrategy::UserIntervention,
            ],
        );

        recovery_strategies.insert(
            ErrorCategory::Configuration,
            vec![
                RecoveryStrategy::ConfigurationReset,
                RecoveryStrategy::UserIntervention,
            ],
        );

        recovery_strategies.insert(
            ErrorCategory::Dependencies,
            vec![
                RecoveryStrategy::Retry,
                RecoveryStrategy::AlternativeSource,
                RecoveryStrategy::Fallback,
            ],
        );

        Self {
            retry_handler: RetryHandler::new(RetryConfig::default()),
            circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60)),
            recovery_strategies,
            auto_recovery_enabled: true,
            metrics: RecoveryMetrics::default(),
        }
    }

    pub fn new_with_config(retry_config: RetryConfig) -> Self {
        let mut manager = Self::new();
        manager.retry_handler = RetryHandler::new(retry_config);
        manager
    }

    pub fn enable_auto_recovery(&mut self, enabled: bool) {
        self.auto_recovery_enabled = enabled;
    }

    pub fn add_recovery_strategy(&mut self, category: ErrorCategory, strategy: RecoveryStrategy) {
        self.recovery_strategies
            .entry(category)
            .or_insert_with(Vec::new)
            .push(strategy);
    }

    /// Execute operation with comprehensive recovery strategies (simplified)
    pub async fn execute_with_recovery<F, Fut, T>(
        &mut self,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, OmniError>>,
    {
        self.metrics.total_errors += 1;

        // Simple execution with retry handler
        match self
            .retry_handler
            .execute_with_context(operation_name, operation)
            .await
        {
            Ok(result) => {
                self.metrics.total_recoveries += 1;
                self.update_success_rate();
                Ok(result)
            }
            Err(error) => {
                // Track error by category
                let category = error.category();
                *self.metrics.errors_by_category.entry(category).or_insert(0) += 1;

                if self.auto_recovery_enabled {
                    match self
                        .attempt_auto_recovery::<T>(&error, operation_name)
                        .await
                    {
                        Ok(_) => Err(error.into()), // Recovery completed but operation needs retry
                        Err(recovery_error) => Err(recovery_error.into()),
                    }
                } else {
                    Err(error.into())
                }
            }
        }
    }

    /// Attempt automatic recovery based on error category
    async fn attempt_auto_recovery<T>(
        &mut self,
        error: &OmniError,
        operation_name: &str,
    ) -> Result<T, OmniError> {
        let category = error.category();
        let strategies = self
            .recovery_strategies
            .get(&category)
            .cloned()
            .unwrap_or_default();

        info!(
            "Attempting automatic recovery for {} error in operation '{}'",
            category, operation_name
        );

        for strategy in strategies {
            info!("Trying recovery strategy: {:?}", strategy);

            match self
                .execute_recovery_strategy(&strategy, error, operation_name)
                .await
            {
                Ok(_) => {
                    info!(
                        "Recovery strategy {:?} succeeded for operation '{}'",
                        strategy, operation_name
                    );
                    *self
                        .metrics
                        .recoveries_by_strategy
                        .entry(strategy.clone())
                        .or_insert(0) += 1;
                    self.metrics.total_recoveries += 1;
                    self.update_success_rate();

                    // Note: In a real implementation, we would re-execute the original operation here
                    // For now, we'll return an error indicating manual retry is needed
                    return Err(OmniError::RecoveryFailed {
                        message: format!(
                            "Recovery strategy {:?} completed, please retry operation",
                            strategy
                        ),
                    });
                }
                Err(recovery_error) => {
                    warn!(
                        "Recovery strategy {:?} failed for operation '{}': {}",
                        strategy, operation_name, recovery_error
                    );
                    continue;
                }
            }
        }

        Err(OmniError::RecoveryFailed {
            message: format!("All recovery strategies failed for {} error", category),
        })
    }

    /// Execute a specific recovery strategy
    async fn execute_recovery_strategy(
        &self,
        strategy: &RecoveryStrategy,
        error: &OmniError,
        _operation_name: &str,
    ) -> Result<()> {
        match strategy {
            RecoveryStrategy::CacheClear => {
                info!("Executing cache clear recovery");
                self.recover_cache_clear().await
            }
            RecoveryStrategy::SystemRepair => {
                info!("Executing system repair recovery");
                self.recover_system_repair().await
            }
            RecoveryStrategy::ConfigurationReset => {
                info!("Executing configuration reset recovery");
                self.recover_configuration_reset().await
            }
            RecoveryStrategy::PermissionEscalation => {
                info!("Executing permission escalation recovery");
                self.recover_permission_escalation().await
            }
            RecoveryStrategy::AlternativeSource => {
                info!("Executing alternative source recovery");
                self.recover_alternative_source().await
            }
            RecoveryStrategy::Retry => {
                info!("Retry strategy already handled by retry handler");
                Ok(())
            }
            RecoveryStrategy::Fallback => {
                info!("Executing fallback recovery");
                self.recover_fallback().await
            }
            RecoveryStrategy::UserIntervention => {
                warn!("User intervention required for error: {}", error);
                Err(anyhow::anyhow!("User intervention required"))
            }
        }
    }

    fn update_success_rate(&mut self) {
        if self.metrics.total_errors > 0 {
            self.metrics.recovery_success_rate =
                (self.metrics.total_recoveries as f64) / (self.metrics.total_errors as f64) * 100.0;
        }
    }

    pub fn get_metrics(&self) -> &RecoveryMetrics {
        &self.metrics
    }

    /// Attempt to recover from a failed operation
    pub async fn attempt_recovery(&self, error: &OmniError) -> Result<()> {
        info!("Attempting recovery for error: {}", error);

        match error {
            OmniError::DatabaseError { .. } => self.recover_database().await,
            OmniError::NetworkError { .. } => self.recover_network().await,
            OmniError::ConfigurationError { .. } => self.recover_configuration().await,
            OmniError::ResourceExhausted { resource } => self.recover_resources(resource).await,
            _ => {
                warn!("No specific recovery method for error type: {}", error);
                Ok(())
            }
        }
    }

    async fn recover_database(&self) -> Result<()> {
        info!("Attempting database recovery");

        // Check database file permissions
        // Attempt to reconnect
        // Clear corrupted cache if necessary

        Ok(())
    }

    async fn recover_network(&self) -> Result<()> {
        info!("Attempting network recovery");

        // Test connectivity
        // Reset network configuration if needed
        // Switch to backup repositories

        Ok(())
    }

    async fn recover_configuration(&self) -> Result<()> {
        info!("Attempting configuration recovery");

        // Validate configuration
        // Reset to defaults if corrupted
        // Backup and restore previous working config

        Ok(())
    }

    async fn recover_resources(&self, resource: &str) -> Result<()> {
        info!("Attempting resource recovery for: {}", resource);

        match resource {
            "disk" => {
                // Clean temporary files
                // Clear caches
                // Free up space
                self.recover_cache_clear().await?;
            }
            "memory" => {
                // Force garbage collection
                // Clear in-memory caches
                // Reduce concurrent operations
            }
            _ => {
                warn!("Unknown resource type for recovery: {}", resource);
            }
        }

        Ok(())
    }

    async fn recover_cache_clear(&self) -> Result<()> {
        info!("Clearing system caches");
        // Implementation would clear package manager caches
        Ok(())
    }

    async fn recover_system_repair(&self) -> Result<()> {
        info!("Performing system repair");
        // Implementation would repair broken package dependencies
        Ok(())
    }

    async fn recover_configuration_reset(&self) -> Result<()> {
        info!("Resetting configuration to defaults");
        // Implementation would reset configurations
        Ok(())
    }

    async fn recover_permission_escalation(&self) -> Result<()> {
        info!("Attempting permission escalation");
        // Implementation would escalate privileges if appropriate
        Ok(())
    }

    async fn recover_alternative_source(&self) -> Result<()> {
        info!("Switching to alternative package source");
        // Implementation would switch to backup repositories
        Ok(())
    }

    async fn recover_fallback(&self) -> Result<()> {
        info!("Executing fallback procedure");
        // Implementation would use fallback methods
        Ok(())
    }
}

/// Error monitoring and metrics collection system
#[derive(Debug, Clone)]
pub struct ErrorMonitor {
    metrics: std::sync::Arc<std::sync::Mutex<ErrorMetrics>>,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    pub errors_by_code: HashMap<String, u64>,
    pub error_rate_per_minute: f64,
    pub last_error_timestamp: u64,
    pub uptime_start: u64,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub error_rate_threshold: f64,
    pub critical_error_threshold: u64,
    pub recovery_failure_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 10.0,       // errors per minute
            critical_error_threshold: 5,      // critical errors before alert
            recovery_failure_threshold: 50.0, // recovery failure rate %
        }
    }
}

impl ErrorMonitor {
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Arc::new(std::sync::Mutex::new(ErrorMetrics {
                uptime_start: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            })),
            alert_thresholds: AlertThresholds::default(),
        }
    }

    pub fn record_error(&self, error: &OmniError) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_errors += 1;

            let category = error.category();
            *metrics.errors_by_category.entry(category).or_insert(0) += 1;

            let severity = error.severity();
            *metrics.errors_by_severity.entry(severity).or_insert(0) += 1;

            let error_code = error.error_code().to_string();
            *metrics.errors_by_code.entry(error_code).or_insert(0) += 1;

            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            metrics.last_error_timestamp = current_time;

            // Calculate error rate per minute
            let uptime_minutes = (current_time - metrics.uptime_start) as f64 / 60.0;
            if uptime_minutes > 0.0 {
                metrics.error_rate_per_minute = metrics.total_errors as f64 / uptime_minutes;
            }

            // Check for alerts
            self.check_alerts(&metrics);
        }
    }

    fn check_alerts(&self, metrics: &ErrorMetrics) {
        // Check error rate threshold
        if metrics.error_rate_per_minute > self.alert_thresholds.error_rate_threshold {
            warn!(
                "High error rate detected: {:.2} errors/minute (threshold: {:.2})",
                metrics.error_rate_per_minute, self.alert_thresholds.error_rate_threshold
            );
        }

        // Check critical error threshold
        if let Some(&critical_count) = metrics.errors_by_severity.get(&ErrorSeverity::Critical) {
            if critical_count >= self.alert_thresholds.critical_error_threshold {
                error!(
                    "Critical error threshold exceeded: {} critical errors (threshold: {})",
                    critical_count, self.alert_thresholds.critical_error_threshold
                );
            }
        }
    }

    pub fn get_metrics(&self) -> ErrorMetrics {
        self.metrics
            .lock()
            .unwrap_or_else(|poisoned| {
                warn!("Error metrics mutex poisoned, returning default");
                poisoned.into_inner()
            })
            .clone()
    }

    pub fn reset_metrics(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            *metrics = ErrorMetrics {
                uptime_start: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            };
        }
    }

    pub fn set_alert_thresholds(&mut self, thresholds: AlertThresholds) {
        self.alert_thresholds = thresholds;
    }
}

/// Global error monitoring instance
static ERROR_MONITOR: std::sync::OnceLock<ErrorMonitor> = std::sync::OnceLock::new();

/// Get the global error monitor instance
pub fn get_error_monitor() -> &'static ErrorMonitor {
    ERROR_MONITOR.get_or_init(|| ErrorMonitor::new())
}

/// Record an error in the global monitor
pub fn record_error(error: &OmniError) {
    get_error_monitor().record_error(error);
}

/// Helper macro for easy error recording and propagation
#[macro_export]
macro_rules! record_and_return_error {
    ($error:expr) => {{
        let error = $error;
        $crate::error_handling::record_error(&error);
        return Err(error);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_omni_error_retryable() {
        assert!(OmniError::NetworkError {
            message: "timeout".to_string(),
            url: None
        }
        .is_retryable());

        assert!(!OmniError::ValidationError {
            field: "package".to_string(),
            message: "invalid".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let security_error = OmniError::SecurityViolation {
            message: "untrusted package".to_string(),
        };
        assert_eq!(security_error.severity(), ErrorSeverity::Critical);

        let network_error = OmniError::NetworkError {
            message: "timeout".to_string(),
            url: None,
        };
        assert_eq!(network_error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new_network();
        assert_eq!(config.max_attempts, 5);
        assert!(config.jitter);

        let critical_config = RetryConfig::new_critical();
        assert_eq!(critical_config.max_attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_handler_success() {
        let handler = RetryHandler::new(RetryConfig::default());
        let mut attempt_count = 0;

        let result = handler
            .execute(|| {
                attempt_count += 1;
                async move {
                    if attempt_count < 2 {
                        Err("temporary failure".to_string())
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count, 2);
    }

    #[tokio::test]
    async fn test_retry_handler_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let handler = RetryHandler::new(config);
        let mut attempt_count = 0;

        let result = handler
            .execute(|| {
                attempt_count += 1;
                async move { Err("persistent failure".to_string()) }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count, 2);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        // Basic creation test - circuit should start closed
    }

    #[test]
    fn test_recovery_manager_creation() {
        let rm = RecoveryManager::new();
        // Should create without panic

        let custom_rm = RecoveryManager::new_with_config(RetryConfig::new_network());
        // Should create with custom config
    }
}
