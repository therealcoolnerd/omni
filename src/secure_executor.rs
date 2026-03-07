use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Clone)]
pub struct SecureExecutor {
    allowed_commands: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub struct ExecutionConfig {
    pub requires_sudo: bool,
    pub timeout: Duration,
    pub allow_network: bool,
    pub working_directory: Option<String>,
    pub environment_vars: HashMap<String, String>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            requires_sudo: false,
            timeout: Duration::from_secs(30),
            allow_network: true,
            working_directory: None,
            environment_vars: HashMap::new(),
        }
    }
}

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
}

impl SecureExecutor {
    pub fn new() -> Result<Self> {
        let mut allowed_commands = HashMap::new();

        // Linux package managers
        allowed_commands.insert("apt".to_string(), vec![
            "install".to_string(), "remove".to_string(), "update".to_string(),
            "upgrade".to_string(), "search".to_string(), "show".to_string(),
            "depends".to_string(), "list".to_string(), "policy".to_string(),
            "-y".to_string(), "--names-only".to_string(), "--upgradable".to_string(),
        ]);
        allowed_commands.insert("dnf".to_string(), vec![
            "install".to_string(), "remove".to_string(), "update".to_string(),
            "check-update".to_string(), "search".to_string(), "info".to_string(),
            "makecache".to_string(), "repoquery".to_string(),
            "-y".to_string(), "--quiet".to_string(), "--requires".to_string(),
        ]);
        allowed_commands.insert("pacman".to_string(), vec![
            "-S".to_string(), "-R".to_string(), "-Sy".to_string(), "-Syu".to_string(),
            "-Ss".to_string(), "-Si".to_string(), "--noconfirm".to_string(),
        ]);
        allowed_commands.insert("snap".to_string(), vec![
            "install".to_string(), "remove".to_string(), "refresh".to_string(),
            "find".to_string(), "info".to_string(), "list".to_string(),
        ]);
        allowed_commands.insert("flatpak".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "update".to_string(),
            "search".to_string(), "info".to_string(), "list".to_string(),
            "-y".to_string(), "--show-runtime".to_string(),
        ]);
        allowed_commands.insert("zypper".to_string(), vec![
            "install".to_string(), "remove".to_string(), "update".to_string(),
            "search".to_string(), "info".to_string(), "refresh".to_string(),
            "-n".to_string(), "--non-interactive".to_string(),
        ]);

        // dpkg tools (used for querying on Debian/Ubuntu)
        allowed_commands.insert("dpkg-query".to_string(), vec![
            "-W".to_string(), "-l".to_string(), "-s".to_string(),
            "--showformat".to_string(),
        ]);
        allowed_commands.insert("dpkg".to_string(), vec![
            "-i".to_string(), "-r".to_string(), "-P".to_string(), "-l".to_string(),
        ]);

        // Windows package managers
        allowed_commands.insert("winget".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "upgrade".to_string(),
            "search".to_string(), "show".to_string(), "list".to_string(),
            "source".to_string(),
            "--id".to_string(), "--silent".to_string(), "--accept-source-agreements".to_string(),
            "--accept-package-agreements".to_string(), "--exact".to_string(),
        ]);
        allowed_commands.insert("choco".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "upgrade".to_string(),
            "search".to_string(), "info".to_string(), "list".to_string(),
            "-y".to_string(), "--yes".to_string(),
        ]);
        allowed_commands.insert("scoop".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "update".to_string(),
            "search".to_string(), "info".to_string(), "list".to_string(),
            "status".to_string(),
        ]);

        // macOS package managers
        allowed_commands.insert("brew".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "upgrade".to_string(),
            "search".to_string(), "info".to_string(), "list".to_string(),
            "update".to_string(), "outdated".to_string(),
        ]);
        allowed_commands.insert("mas".to_string(), vec![
            "install".to_string(), "uninstall".to_string(), "upgrade".to_string(),
            "search".to_string(), "info".to_string(), "list".to_string(),
            "outdated".to_string(),
        ]);

        // System utilities
        allowed_commands.insert("wget".to_string(), vec![
            "-O".to_string(), "-q".to_string(), "--timeout".to_string(),
        ]);
        allowed_commands.insert("curl".to_string(), vec![
            "-o".to_string(), "-s".to_string(), "--max-time".to_string(), "-L".to_string(),
        ]);
        allowed_commands.insert("gpg".to_string(), vec![
            "--import".to_string(), "--verify".to_string(), "--keyserver".to_string(),
        ]);

        // Hardware detection
        allowed_commands.insert("lspci".to_string(), vec!["-nn".to_string(), "-v".to_string()]);
        allowed_commands.insert("lsusb".to_string(), vec!["-v".to_string()]);
        allowed_commands.insert("dmidecode".to_string(), vec!["-s".to_string(), "-t".to_string()]);

        Ok(Self { allowed_commands })
    }

    pub async fn execute_package_command(
        &self,
        command: &str,
        args: &[&str],
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // Validate command is allowed
        self.validate_command(command, args)?;
        
        // Sanitize arguments
        let sanitized_args = self.sanitize_arguments(args)?;
        
        info!("Executing command: {} with args: {:?}", command, sanitized_args);
        
        // Execute with timeout using tokio
        let output = tokio::time::timeout(config.timeout, async {
            let output = tokio::process::Command::new(command)
                .args(&sanitized_args)
                .current_dir(config.working_directory.as_deref().unwrap_or("."))
                .kill_on_drop(true)
                .output()
                .await?;
            Ok::<_, anyhow::Error>(output)
        })
        .await
        .map_err(|_| anyhow!("Command execution timed out after {:?}", config.timeout))?;
        
        let output = output?;
        let execution_time = start_time.elapsed();
        
        let result = ExecutionResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            execution_time,
        };
        
        // Log execution details
        info!("Command executed in {:?}, exit code: {}", execution_time, result.exit_code);
        
        if result.exit_code != 0 {
            warn!("Command failed with exit code: {}", result.exit_code);
            warn!("Stderr: {}", result.stderr);
        }
        
        Ok(result)
    }
    
    fn validate_command(&self, command: &str, args: &[&str]) -> Result<()> {
        let allowed_args = self
            .allowed_commands
            .get(command)
            .ok_or_else(|| anyhow!("Command '{}' not in allowed list", command))?;

        // Validate the first argument (subcommand or primary flag). Every call
        // must have a leading operation so we know what it intends to do.
        let first_arg = args
            .first()
            .ok_or_else(|| anyhow!("No arguments provided for command '{}'", command))?;

        if !allowed_args.contains(&first_arg.to_string()) {
            return Err(anyhow!(
                "Operation '{}' not allowed for command '{}'",
                first_arg,
                command
            ));
        }

        // Validate any additional flags (args starting with '-') in the rest of
        // the argument list. Package names and other positional values are handled
        // by sanitize_arguments instead.
        for arg in args.iter().skip(1) {
            if arg.starts_with('-') && !allowed_args.contains(&arg.to_string()) {
                // Allow flags that contain '=' (e.g. --showformat=${Version})
                // by checking only the flag name portion.
                let flag_name = arg.splitn(2, '=').next().unwrap_or(arg);
                if !allowed_args.contains(&flag_name.to_string()) {
                    return Err(anyhow!(
                        "Flag '{}' not allowed for command '{}'",
                        arg,
                        command
                    ));
                }
            }
        }

        Ok(())
    }
    
    fn sanitize_arguments(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut sanitized = Vec::new();
        
        for arg in args {
            // Check for dangerous characters
            if arg.contains(';') || arg.contains('|') || arg.contains('&') || arg.contains('`') {
                return Err(anyhow!("Dangerous characters found in argument: {}", arg));
            }
            
            // Check for command substitution
            if arg.contains("$(") || arg.contains("${") {
                return Err(anyhow!("Command substitution not allowed in argument: {}", arg));
            }
            
            // Check for path traversal
            if arg.contains("../") || arg.contains("..\\") {
                return Err(anyhow!("Path traversal not allowed in argument: {}", arg));
            }
            
            // Limit argument length
            if arg.len() > 1024 {
                return Err(anyhow!("Argument too long: {}", arg));
            }
            
            sanitized.push(arg.to_string());
        }
        
        Ok(sanitized)
    }
    
    /// Add a new allowed command with its permitted arguments
    pub fn add_allowed_command(&mut self, command: String, allowed_args: Vec<String>) {
        self.allowed_commands.insert(command, allowed_args);
    }
    
    /// Check if a command is allowed
    pub fn is_command_allowed(&self, command: &str) -> bool {
        self.allowed_commands.contains_key(command)
    }
}
