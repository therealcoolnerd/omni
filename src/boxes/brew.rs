use std::process::Command;
use anyhow::{Result, anyhow};
use crate::distro::PackageManager;

pub struct BrewBox;

impl BrewBox {
    pub fn new() -> Self {
        BrewBox
    }
    
    pub fn is_available() -> bool {
        Command::new("brew")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for BrewBox {
    fn install(&self, package: &str) -> Result<()> {
        let output = Command::new("brew")
            .args(&["install", package])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew install failed: {}", stderr))
        }
    }
    
    fn remove(&self, package: &str) -> Result<()> {
        let output = Command::new("brew")
            .args(&["uninstall", package])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew uninstall failed: {}", stderr))
        }
    }
    
    fn update(&self, package: Option<&str>) -> Result<()> {
        // First update brew itself
        let _ = Command::new("brew")
            .args(&["update"])
            .output()?;
            
        // Then upgrade packages
        let mut args = vec!["upgrade"];
        
        if let Some(pkg) = package {
            args.push(pkg);
        }
        
        let output = Command::new("brew")
            .args(&args)
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew upgrade failed: {}", stderr))
        }
    }
    
    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = Command::new("brew")
            .args(&["search", query])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("==>") {
                        Some(trimmed.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew search failed: {}", stderr))
        }
    }
    
    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("brew")
            .args(&["list"])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew list failed: {}", stderr))
        }
    }
    
    fn get_info(&self, package: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(&["info", package])
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("brew info failed: {}", stderr))
        }
    }
    
    fn needs_privilege(&self) -> bool {
        false // Homebrew doesn't require admin privileges
    }
    
    fn get_name(&self) -> &'static str {
        "brew"
    }
    
    fn get_priority(&self) -> u8 {
        80 // High priority for macOS systems
    }
}