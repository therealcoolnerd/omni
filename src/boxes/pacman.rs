use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

/// Secure Pacman package manager wrapper
pub struct PacmanBox {
    executor: SecureExecutor,
}

impl PacmanBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("pacman")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for PacmanBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via pacman", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("pacman", &["-S", "--noconfirm", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Pacman successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Pacman failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "pacman".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via pacman", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("pacman", &["-R", "--noconfirm", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Pacman successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Pacman failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "pacman".to_string(),
                    reason: format!("Remove failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            if let Some(pkg) = package {
                // Update specific package
                info!("Upgrading '{}' via pacman", pkg);
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(600),
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("pacman", &["-S", "--noconfirm", pkg], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Pacman upgrade completed successfully");
                    Ok(())
                } else {
                    error!("❌ Pacman upgrade failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: pkg.to_string(),
                        box_type: "pacman".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            } else {
                // Full system update
                info!("Updating all packages via pacman");
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(1200), // 20 minutes for updates
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("pacman", &["-Syu", "--noconfirm"], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Pacman system update completed successfully");
                    Ok(())
                } else {
                    error!("❌ Pacman system update failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: "all".to_string(),
                        box_type: "pacman".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via pacman", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("pacman", &["-Ss", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.starts_with("    ") {
                            // Skip description lines
                            None
                        } else if line.contains("/") {
                            // Package name line
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if !parts.is_empty() {
                                let pkg_name = parts[0].split('/').nth(1).unwrap_or(parts[0]);
                                Some(pkg_name.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                info!("✅ Found {} packages matching '{}'", packages.len(), query);
                Ok(packages)
            } else {
                error!("❌ Pacman search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via pacman");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("pacman", &["-Q"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            Some(parts[0].to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Pacman list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "pacman".to_string(),
                    reason: format!("List failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Getting info for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("pacman", &["-Si", package], config)
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                Err(OmniError::PackageNotFound {
                    package: package.to_string(),
                }
                .into())
            }
        })
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        let package = package.to_string();
        let executor = Arc::clone(&self.executor);
        
        RuntimeManager::block_on(async move {
            info!("Getting installed version for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("pacman", &["-Q", &package], config)
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse "package_name version" format
                if let Some(version_part) = result.stdout.trim().split_whitespace().nth(1) {
                    let version = version_part.to_string();
                    info!("✅ Found installed version '{}' for package '{}'", version, package);
                    Ok(Some(version))
                } else {
                    info!("ℹ️ Package '{}' output format unexpected: {}", package, result.stdout.trim());
                    Ok(None)
                }
            } else {
                info!("ℹ️ Package '{}' is not installed", package);
                Ok(None)
            }
        })
    }

    fn needs_privilege(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        "pacman"
    }

    fn get_priority(&self) -> u8 {
        90 // Very high priority for Arch systems
    }
}

// Legacy functions for backward compatibility
pub fn install_with_pacman(package: &str) {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        match PacmanBox::new() {
            Ok(manager) => {
                if let Err(e) = manager.install(package) {
                    error!("Pacman installation failed: {}", e);
                }
            }
            Err(e) => error!("Failed to create Pacman manager: {}", e),
        }
    });
}

pub fn uninstall_with_pacman(package: &str) {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        match PacmanBox::new() {
            Ok(manager) => {
                if let Err(e) = manager.remove(package) {
                    error!("Pacman removal failed: {}", e);
                }
            }
            Err(e) => error!("Failed to create Pacman manager: {}", e),
        }
    });
}
