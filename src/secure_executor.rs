use crate::error_handling::{OmniError, RetryConfig, RetryHandler};
use crate::input_validation::InputValidator;
use crate::privilege_manager::PrivilegeManager;
use crate::security::{SecurityPolicy, SecurityVerifier};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{error, info, warn};

/// Secure command execution wrapper that validates inputs and manages privileges
#[derive(Debug, Clone)]
pub struct SecureExecutor {
    privilege_manager: PrivilegeManager,
    security_verifier: SecurityVerifier,
    retry_handler: RetryHandler,
    allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub timeout: Duration,
    pub requires_sudo: bool,
    pub validate_output: bool,
    pub sandbox: bool,
    pub max_retries: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            requires_sudo: false,
            validate_output: true,
            sandbox: true,
            max_retries: 3,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration: Duration,
    pub was_retried: bool,
}

impl SecureExecutor {
    pub fn new() -> Result<Self> {
        let mut privilege_manager = PrivilegeManager::new();
        privilege_manager.store_credentials();

        let security_policy = SecurityPolicy::default();
        let security_verifier = SecurityVerifier::new(security_policy);

        let retry_config = RetryConfig::default();
        let retry_handler = RetryHandler::new(retry_config);

        // Define allowed package manager commands
        let allowed_commands = vec![
            "apt".to_string(),
            "apt-get".to_string(),
            "dnf".to_string(),
            "yum".to_string(),
            "pacman".to_string(),
            "snap".to_string(),
            "flatpak".to_string(),
            "dpkg".to_string(),
            "rpm".to_string(),
            "dpkg-query".to_string(),
            "rpm-query".to_string(),
            // Add other safe commands as needed
        ];

        Ok(Self {
            privilege_manager,
            security_verifier,
            retry_handler,
            allowed_commands,
        })
    }

    /// Execute a package manager command securely with full validation
    pub async fn execute_package_command(
        &self,
        command: &str,
        args: &[&str],
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        info!("Executing secure package command: {} {:?}", command, args);

        // Step 1: Validate command is allowed
        self.validate_command(command)?;

        // Step 2: Validate all arguments
        let validated_args: Vec<String> = args
            .iter()
            .map(|arg| self.validate_argument(arg))
            .collect::<Result<Vec<_>>>()?;

        // Step 3: Check privileges
        if config.requires_sudo && !PrivilegeManager::is_root() && !PrivilegeManager::can_sudo() {
            return Err(OmniError::PermissionDenied {
                operation: format!("{} {}", command, args.join(" ")),
            }
            .into());
        }

        // Step 4: Execute with retry logic
        let start_time = std::time::Instant::now();
        let mut was_retried = false;

        let result = self
            .retry_handler
            .execute(|| async {
                was_retried = true;
                self.execute_with_safety_checks(command, &validated_args, &config)
                    .await
                    .map_err(|e| crate::error_handling::OmniError::ValidationError {
                        field: "command_execution".to_string(),
                        message: e.to_string(),
                    })
            })
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let duration = start_time.elapsed();

        Ok(ExecutionResult {
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            exit_code: result.status.code().unwrap_or(-1),
            duration,
            was_retried,
        })
    }

    /// Execute installation with security verification
    pub async fn secure_install(
        &self,
        package_name: &str,
        box_type: &str,
        source_url: Option<&str>,
    ) -> Result<ExecutionResult> {
        info!(
            "Starting secure installation of {} via {}",
            package_name, box_type
        );

        // Validate package name
        InputValidator::validate_package_name(package_name)?;
        InputValidator::validate_box_type(box_type)?;

        // Validate URL if provided
        if let Some(url) = source_url {
            InputValidator::validate_url(url)?;
        }

        // Create execution config based on package manager
        let config = self.create_config_for_box_type(box_type)?;

        // Execute the installation
        match box_type {
            "apt" => self.execute_apt_install(package_name, config).await,
            "dnf" => self.execute_dnf_install(package_name, config).await,
            "pacman" => self.execute_pacman_install(package_name, config).await,
            "snap" => self.execute_snap_install(package_name, config).await,
            "flatpak" => self.execute_flatpak_install(package_name, config).await,
            _ => Err(OmniError::UnsupportedBoxType {
                box_type: box_type.to_string(),
            }
            .into()),
        }
    }

    fn validate_command(&self, command: &str) -> Result<()> {
        if !self.allowed_commands.contains(&command.to_string()) {
            return Err(OmniError::SecurityViolation {
                message: format!("Command '{}' is not in allowed list", command),
            }
            .into());
        }

        // Additional validation
        InputValidator::validate_shell_safe(command)?;

        Ok(())
    }

    fn validate_argument(&self, arg: &str) -> Result<String> {
        // Strict validation for command arguments
        InputValidator::validate_shell_safe(arg)?;

        // Additional package manager specific validation
        if arg.starts_with('-') {
            // This is a flag - validate it's a known safe flag
            self.validate_flag(arg)?;
        } else if arg.contains('=') {
            // This might be a key=value parameter
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() == 2 {
                InputValidator::validate_shell_safe(parts[0])?;
                InputValidator::validate_shell_safe(parts[1])?;
            }
        }

        Ok(arg.to_string())
    }

    fn validate_flag(&self, flag: &str) -> Result<()> {
        // Common safe flags for package managers
        let safe_flags = [
            "-y",
            "--yes",
            "--assume-yes",
            "-q",
            "--quiet",
            "-v",
            "--verbose",
            "--no-confirm",
            "--noconfirm",
            "--force-yes", // Use with caution
            "--dry-run",
            "--simulate",
            "-s",
            "--search",
            "-i",
            "--info",
            "-u",
            "--update",
            "-h",
            "--help",
            "--version",
            "-R",
            "-S",
            "-Q", // pacman specific
            "--user",
            "--system", // flatpak specific
            "install",
            "remove",
            "update",
            "upgrade",
            "search",
            "info",
        ];

        if !safe_flags.contains(&flag) {
            warn!("Potentially unsafe flag detected: {}", flag);
            // Allow but log for review
        }

        Ok(())
    }

    async fn execute_with_safety_checks(
        &self,
        command: &str,
        args: &[String],
        config: &ExecutionConfig,
    ) -> Result<std::process::Output> {
        // Create sandbox if requested
        let _sandbox_guard = if config.sandbox {
            let temp_dir = tempfile::tempdir()?;
            Some(self.privilege_manager.create_sandbox(temp_dir.path())?)
        } else {
            None
        };

        // Build command
        let mut cmd = if config.requires_sudo && !PrivilegeManager::is_root() {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("-n"); // Non-interactive
            sudo_cmd.arg(command);
            sudo_cmd.args(args);
            sudo_cmd
        } else {
            let mut cmd = Command::new(command);
            cmd.args(args);
            cmd
        };

        // Set security constraints
        cmd.stdin(Stdio::null()); // No interactive input
        cmd.env_remove("LD_PRELOAD"); // Prevent library injection
        cmd.env_remove("LD_LIBRARY_PATH"); // Prevent library path manipulation

        // Set timeout
        let output = tokio::time::timeout(config.timeout, async {
            tokio::task::spawn_blocking(move || cmd.output()).await?
        })
        .await
        .map_err(|_| OmniError::TimeoutError {
            operation: format!("{} {}", command, args.join(" ")),
            duration: config.timeout,
        })??;

        // Validate output if requested
        if config.validate_output {
            self.validate_command_output(&output)?;
        }

        Ok(output)
    }

    fn validate_command_output(&self, output: &std::process::Output) -> Result<()> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for suspicious patterns in output
        let suspicious_patterns = [
            "rm -rf",
            "/etc/passwd",
            "/etc/shadow",
            "chmod 777",
            "curl http://",
            "wget http://",
            "nc -l",
            "netcat -l",
            "/bin/sh",
            "/bin/bash",
            "$(",
            "`",
        ];

        for pattern in &suspicious_patterns {
            if stdout.contains(pattern) || stderr.contains(pattern) {
                warn!("Suspicious pattern detected in command output: {}", pattern);
                // Log but don't fail - might be legitimate
            }
        }

        Ok(())
    }

    fn create_config_for_box_type(&self, box_type: &str) -> Result<ExecutionConfig> {
        match box_type {
            "apt" | "dnf" | "pacman" => Ok(ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600), // 10 minutes for installations
                ..ExecutionConfig::default()
            }),
            "snap" => Ok(ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(900), // 15 minutes for snaps
                ..ExecutionConfig::default()
            }),
            "flatpak" => Ok(ExecutionConfig {
                requires_sudo: false, // Can install user-wide
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            }),
            _ => Err(OmniError::UnsupportedBoxType {
                box_type: box_type.to_string(),
            }
            .into()),
        }
    }

    async fn execute_apt_install(
        &self,
        package: &str,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        // Update package lists first
        let update_result = self
            .execute_package_command(
                "apt",
                &["update"],
                ExecutionConfig {
                    requires_sudo: true,
                    ..config.clone()
                },
            )
            .await;

        if let Err(e) = update_result {
            warn!("Failed to update apt cache: {}", e);
        }

        // Install package
        self.execute_package_command("apt", &["install", "-y", package], config)
            .await
    }

    async fn execute_dnf_install(
        &self,
        package: &str,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        self.execute_package_command("dnf", &["install", "-y", package], config)
            .await
    }

    async fn execute_pacman_install(
        &self,
        package: &str,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        // Sync package databases first
        let sync_result = self
            .execute_package_command(
                "pacman",
                &["-Sy"],
                ExecutionConfig {
                    requires_sudo: true,
                    ..config.clone()
                },
            )
            .await;

        if let Err(e) = sync_result {
            warn!("Failed to sync pacman databases: {}", e);
        }

        // Install package
        self.execute_package_command("pacman", &["-S", "--noconfirm", package], config)
            .await
    }

    async fn execute_snap_install(
        &self,
        package: &str,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        self.execute_package_command("snap", &["install", package], config)
            .await
    }

    async fn execute_flatpak_install(
        &self,
        package: &str,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        self.execute_package_command("flatpak", &["install", "-y", package], config)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secure_executor_creation() {
        let executor = SecureExecutor::new();
        assert!(executor.is_ok());
    }

    #[test]
    fn test_command_validation() {
        let executor = SecureExecutor::new().unwrap();

        // Valid commands
        assert!(executor.validate_command("apt").is_ok());
        assert!(executor.validate_command("dnf").is_ok());
        assert!(executor.validate_command("pacman").is_ok());

        // Invalid commands
        assert!(executor.validate_command("rm").is_err());
        assert!(executor.validate_command("curl").is_err());
        assert!(executor.validate_command("bash").is_err());
    }

    #[test]
    fn test_argument_validation() {
        let executor = SecureExecutor::new().unwrap();

        // Valid arguments
        assert!(executor.validate_argument("firefox").is_ok());
        assert!(executor.validate_argument("-y").is_ok());
        assert!(executor.validate_argument("--quiet").is_ok());

        // Invalid arguments
        assert!(executor.validate_argument("test; rm -rf /").is_err());
        assert!(executor.validate_argument("$(whoami)").is_err());
        assert!(executor.validate_argument("package && malicious").is_err());
    }

    #[test]
    fn test_flag_validation() {
        let executor = SecureExecutor::new().unwrap();

        // Safe flags
        assert!(executor.validate_flag("-y").is_ok());
        assert!(executor.validate_flag("--yes").is_ok());
        assert!(executor.validate_flag("--quiet").is_ok());
        assert!(executor.validate_flag("--noconfirm").is_ok());

        // Potentially unsafe flags (logged but allowed)
        assert!(executor.validate_flag("--dangerous-flag").is_ok());
    }
}
