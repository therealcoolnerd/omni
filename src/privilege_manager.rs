use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

pub struct PrivilegeManager {
    has_sudo: bool,
    is_root_user: bool,
}

impl PrivilegeManager {
    pub fn new() -> Self {
        let is_root_user = Self::check_is_root();
        let has_sudo = if !is_root_user {
            Self::check_sudo_access()
        } else {
            true
        };

        Self {
            has_sudo,
            is_root_user,
        }
    }

    pub fn store_credentials(&mut self) {
        // For now, just refresh the sudo timestamp if we have sudo access
        if self.has_sudo && !self.is_root_user {
            let _ = Command::new("sudo")
                .args(&["-v"])
                .output();
        }
    }

    pub fn is_root() -> bool {
        Self::check_is_root()
    }

    pub fn can_sudo() -> bool {
        Self::check_sudo_access()
    }

    pub fn validate_minimal_privileges() -> Result<()> {
        // Check if we can run basic commands
        let output = Command::new("id")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Cannot execute basic commands"));
        }

        Ok(())
    }

    pub fn execute_with_sudo(&self, command: &str, args: &[&str]) -> Result<()> {
        if self.is_root_user {
            // Already root, execute directly
            let output = Command::new(command)
                .args(args)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Command failed: {}", stderr));
            }
        } else if self.has_sudo {
            // Use sudo
            let mut sudo_args = vec!["sudo"];
            sudo_args.push(command);
            sudo_args.extend(args);

            let output = Command::new("sudo")
                .args(&[command])
                .args(args)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Sudo command failed: {}", stderr));
            }
        } else {
            return Err(anyhow::anyhow!("Insufficient privileges to execute command"));
        }

        Ok(())
    }

    pub fn execute_with_sudo_output(&self, command: &str, args: &[&str]) -> Result<String> {
        if self.is_root_user {
            // Already root, execute directly
            let output = Command::new(command)
                .args(args)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Command failed: {}", stderr);
            }

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else if self.has_sudo {
            // Use sudo
            let output = Command::new("sudo")
                .args(&[command])
                .args(args)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Sudo command failed: {}", stderr);
            }

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Insufficient privileges to execute command"))
        }
    }

    fn check_is_root() -> bool {
        // Check if current user is root (UID 0)
        unsafe {
            libc::getuid() == 0
        }
    }

    fn check_sudo_access() -> bool {
        // Try to run sudo -n true to check if we have sudo access without password
        let output = Command::new("sudo")
            .args(&["-n", "true"])
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}
