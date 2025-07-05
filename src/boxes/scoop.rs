use crate::distro::PackageManager;
use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::{info, warn};

pub struct ScoopBox;

impl ScoopBox {
    pub fn new() -> Result<Self> {
        Ok(ScoopBox)
    }

    pub fn is_available() -> bool {
        Command::new("scoop")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for ScoopBox {
    fn install(&self, package: &str) -> Result<()> {
        let output = Command::new("scoop").args(&["install", package]).output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("scoop install failed: {}", stderr))
        }
    }

    fn remove(&self, package: &str) -> Result<()> {
        let output = Command::new("scoop")
            .args(&["uninstall", package])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("scoop uninstall failed: {}", stderr))
        }
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        // First update scoop itself and buckets
        let _ = Command::new("scoop").args(&["update"]).output()?;

        // Then update packages
        let mut args = vec!["update"];

        if let Some(pkg) = package {
            args.push(pkg);
        } else {
            args.push("*");
        }

        let output = Command::new("scoop").args(&args).output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("scoop update failed: {}", stderr))
        }
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = Command::new("scoop").args(&["search", query]).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    // Scoop search output format: "bucket/package (version)"
                    if line.contains("(") && !line.starts_with("'") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            // Extract package name, handling bucket/package format
                            let package_full = parts[0];
                            if package_full.contains("/") {
                                let package_parts: Vec<&str> = package_full.split("/").collect();
                                if package_parts.len() >= 2 {
                                    Some(package_parts[1].to_string())
                                } else {
                                    Some(package_full.to_string())
                                }
                            } else {
                                Some(package_full.to_string())
                            }
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
            Err(anyhow!("scoop search failed: {}", stderr))
        }
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("scoop").args(&["list"]).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2
                        && !line.starts_with("Installed")
                        && !line.starts_with("Name")
                    {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("scoop list failed: {}", stderr))
        }
    }

    fn get_info(&self, package: &str) -> Result<String> {
        let output = Command::new("scoop").args(&["info", package]).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("scoop info failed: {}", stderr))
        }
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        info!("Getting installed version for package '{}'", package);

        let output = Command::new("scoop")
            .args(&["list", package])
            .output()
            .map_err(|e| anyhow!("Failed to execute scoop command: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Parse scoop list output
            for line in stdout.lines() {
                if line.contains(package) && !line.starts_with("Name") && !line.starts_with("-") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == package {
                        let version = parts[1].to_string();
                        info!(
                            "✅ Found installed version '{}' for package '{}'",
                            version, package
                        );
                        return Ok(Some(version));
                    }
                }
            }
            info!(
                "ℹ️ Package '{}' output format unexpected: {}",
                package,
                stdout.trim()
            );
            Ok(None)
        } else {
            info!("ℹ️ Package '{}' is not installed", package);
            Ok(None)
        }
    }

    fn needs_privilege(&self) -> bool {
        false // Scoop doesn't require admin privileges
    }

    fn get_name(&self) -> &'static str {
        "scoop"
    }

    fn get_priority(&self) -> u8 {
        60 // Medium priority for Windows systems
    }
}
