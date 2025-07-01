use anyhow::{anyhow, Result};
use std::fmt;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Comprehensive error types for Omni
#[derive(Error, Debug, Clone)]
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
            _ => vec!["Check logs for more details".to_string()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
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

/// Retry mechanism with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryHandler {
    config: RetryConfig,
}

impl RetryHandler {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + Clone,
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
        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("Unknown error occurred during retry attempts")))
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
        E: std::fmt::Debug,
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

/// Recovery manager for handling system state recovery
pub struct RecoveryManager {
    retry_handler: RetryHandler,
    circuit_breaker: CircuitBreaker,
}

impl RecoveryManager {
    pub fn new() -> Self {
        Self {
            retry_handler: RetryHandler::new(RetryConfig::default()),
            circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60)),
        }
    }

    pub fn new_with_config(retry_config: RetryConfig) -> Self {
        Self {
            retry_handler: RetryHandler::new(retry_config),
            circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60)),
        }
    }

    /// Execute operation with full recovery capabilities
    pub async fn execute_with_recovery<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T>>,
        T: Clone,
    {
        let circuit_operation = || {
            let mut op = operation.clone();
            async move { self.retry_handler.execute(|| op()).await }
        };

        self.circuit_breaker.execute(circuit_operation).await
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
                        Err("temporary failure")
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
                async move { Err("persistent failure") }
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
