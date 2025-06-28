use std::process::Command;
use anyhow::{Result, anyhow};
use crate::distro::PackageManager;

pub struct WingetBox;

impl WingetBox {
    pub fn new() -> Self {
        WingetBox
    }
    
    pub fn is_available() -> bool {
        Command::new("winget")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for WingetBox {
    fn install(&self, package: &str) -> Result<()> {
        let output = Command::new("winget")
            .args(&["install", package, "--accept-package-agreements", "--accept-source-agreements"])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget install failed: {}", stderr))
        }
    }
    
    fn remove(&self, package: &str) -> Result<()> {
        let output = Command::new("winget")
            .args(&["uninstall", package])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget uninstall failed: {}", stderr))
        }
    }
    
    fn update(&self, package: Option<&str>) -> Result<()> {
        let mut args = vec!["upgrade"];
        
        if let Some(pkg) = package {
            args.push(pkg);
        } else {
            args.push("--all");
        }
        args.extend(&["--accept-package-agreements", "--accept-source-agreements"]);
        
        let output = Command::new("winget")
            .args(&args)
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget upgrade failed: {}", stderr))
        }
    }
    
    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = Command::new("winget")
            .args(&["search", query])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .skip(2) // Skip header lines
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget search failed: {}", stderr))
        }
    }
    
    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("winget")
            .args(&["list"])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .skip(2) // Skip header lines
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget list failed: {}", stderr))
        }
    }
    
    fn get_info(&self, package: &str) -> Result<String> {
        let output = Command::new("winget")
            .args(&["show", package])
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("winget show failed: {}", stderr))
        }
    }
    
    fn needs_privilege(&self) -> bool {
        false // winget typically doesn't require admin privileges
    }
    
    fn get_name(&self) -> &'static str {
        "winget"
    }
    
    fn get_priority(&self) -> u8 {
        80 // High priority for Windows systems
    }
}