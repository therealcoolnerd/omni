use std::process::Command;
use anyhow::{Result, anyhow};
use crate::distro::PackageManager;

pub struct MasBox;

impl MasBox {
    pub fn new() -> Result<Self> {
        Ok(MasBox)
    }
    
    pub fn is_available() -> bool {
        Command::new("mas")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for MasBox {
    fn install(&self, package: &str) -> Result<()> {
        // mas requires app ID, try to install by ID first, then by name
        let output = if package.chars().all(char::is_numeric) {
            // Install by App Store ID
            Command::new("mas")
                .args(&["install", package])
                .output()?
        } else {
            // Search for app name and install first result
            let search_output = Command::new("mas")
                .args(&["search", package])
                .output()?;
                
            if search_output.status.success() {
                let stdout = String::from_utf8_lossy(&search_output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let app_id = parts[0];
                        Command::new("mas")
                            .args(&["install", app_id])
                            .output()?
                    } else {
                        return Err(anyhow!("No app found for: {}", package));
                    }
                } else {
                    return Err(anyhow!("No app found for: {}", package));
                }
            } else {
                return Err(anyhow!("Search failed for: {}", package));
            }
        };
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("mas install failed: {}", stderr))
        }
    }
    
    fn remove(&self, package: &str) -> Result<()> {
        // mas doesn't support uninstall directly
        Err(anyhow!("mas doesn't support uninstalling apps. Use Finder to delete apps manually."))
    }
    
    fn update(&self, package: Option<&str>) -> Result<()> {
        let mut args = vec!["upgrade"];
        
        if let Some(pkg) = package {
            args.push(pkg);
        }
        
        let output = Command::new("mas")
            .args(&args)
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("mas upgrade failed: {}", stderr))
        }
    }
    
    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = Command::new("mas")
            .args(&["search", query])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        // Format: "ID App Name (Version)"
                        let app_info = parts[1].trim();
                        if let Some(name_end) = app_info.find(" (") {
                            Some(format!("{} - {}", parts[0], &app_info[..name_end]))
                        } else {
                            Some(format!("{} - {}", parts[0], app_info))
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("mas search failed: {}", stderr))
        }
    }
    
    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("mas")
            .args(&["list"])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        // Format: "ID App Name (Version)"
                        let app_info = parts[1].trim();
                        if let Some(name_end) = app_info.find(" (") {
                            Some(format!("{} - {}", parts[0], &app_info[..name_end]))
                        } else {
                            Some(format!("{} - {}", parts[0], app_info))
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("mas list failed: {}", stderr))
        }
    }
    
    fn get_info(&self, package: &str) -> Result<String> {
        // mas doesn't have a direct info command, use search
        let output = Command::new("mas")
            .args(&["search", package])
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("mas search failed: {}", stderr))
        }
    }
    
    fn needs_privilege(&self) -> bool {
        false // mas doesn't require admin privileges, but needs Apple ID login
    }
    
    fn get_name(&self) -> &'static str {
        "mas"
    }
    
    fn get_priority(&self) -> u8 {
        60 // Medium priority for macOS systems
    }
}