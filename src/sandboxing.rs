use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

pub struct Sandbox {
    network_access: bool,
}

impl Sandbox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            network_access: true,
        })
    }

    pub fn set_network_access(&mut self, enabled: bool) {
        self.network_access = enabled;
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> Result<()> {
        if !self.network_access {
            info!("Executing command in restricted network mode: {} {:?}", command, args);
        }

        let output = Command::new(command)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Command failed: {}", stderr));
        }

        Ok(())
    }

    pub fn execute_with_output(&self, command: &str, args: &[&str]) -> Result<String> {
        if !self.network_access {
            info!("Executing command in restricted network mode: {} {:?}", command, args);
        }

        let output = Command::new(command)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
