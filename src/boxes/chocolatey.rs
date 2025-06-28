use std::process::Command;
use anyhow::{Result, anyhow};
use crate::distro::PackageManager;

pub struct ChocolateyBox;

impl ChocolateyBox {
    pub fn new() -> Self {
        ChocolateyBox
    }
    
    pub fn is_available() -> bool {
        Command::new("choco")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for ChocolateyBox {
    fn install(&self, package: &str) -> Result<()> {
        let output = Command::new("choco")
            .args(&["install", package, "-y"])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey install failed: {}", stderr))
        }
    }
    
    fn remove(&self, package: &str) -> Result<()> {
        let output = Command::new("choco")
            .args(&["uninstall", package, "-y"])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey uninstall failed: {}", stderr))
        }
    }
    
    fn update(&self, package: Option<&str>) -> Result<()> {
        let mut args = vec!["upgrade"];
        
        if let Some(pkg) = package {
            args.push(pkg);
        } else {
            args.push("all");
        }
        args.push("-y");
        
        let output = Command::new("choco")
            .args(&args)
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey upgrade failed: {}", stderr))
        }
    }
    
    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = Command::new("choco")
            .args(&["search", query])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    if line.contains(" | ") && !line.starts_with("Chocolatey") {
                        let parts: Vec<&str> = line.split(" | ").collect();
                        if !parts.is_empty() {
                            Some(parts[0].trim().to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey search failed: {}", stderr))
        }
    }
    
    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("choco")
            .args(&["list", "--local-only"])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    if line.contains(" ") && !line.starts_with("Chocolatey") && !line.contains("packages installed") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            Some(parts[0].to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey list failed: {}", stderr))
        }
    }
    
    fn get_info(&self, package: &str) -> Result<String> {
        let output = Command::new("choco")
            .args(&["info", package])
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("chocolatey info failed: {}", stderr))
        }
    }
    
    fn needs_privilege(&self) -> bool {
        true // Chocolatey typically requires admin privileges
    }
    
    fn get_name(&self) -> &'static str {
        "chocolatey"
    }
    
    fn get_priority(&self) -> u8 {
        70 // Medium-high priority for Windows systems
    }
}