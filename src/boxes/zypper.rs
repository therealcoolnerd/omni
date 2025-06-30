use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info};

/// Secure Zypper package manager wrapper for openSUSE
pub struct ZypperBox {
    executor: SecureExecutor,
}

impl ZypperBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("zypper")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for ZypperBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via zypper", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("zypper", &["install", "-y", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Zypper successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Zypper failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "zypper".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via zypper", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("zypper", &["remove", "-y", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Zypper successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Zypper failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "zypper".to_string(),
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
                info!("Upgrading '{}' via zypper", pkg);
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(600),
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("zypper", &["update", "-y", pkg], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Zypper upgrade completed successfully");
                    Ok(())
                } else {
                    error!("❌ Zypper upgrade failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: pkg.to_string(),
                        box_type: "zypper".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            } else {
                // Full system update
                info!("Updating all packages via zypper");
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(1200), // 20 minutes for updates
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("zypper", &["update", "-y"], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Zypper system update completed successfully");
                    Ok(())
                } else {
                    error!("❌ Zypper system update failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: "all".to_string(),
                        box_type: "zypper".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via zypper", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("zypper", &["search", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.starts_with("| ") && line.contains(" | ") {
                            let parts: Vec<&str> = line.split(" | ").collect();
                            if parts.len() >= 2 {
                                let name = parts[1].trim();
                                if !name.is_empty() && name != "Name" {
                                    Some(name.to_string())
                                } else {
                                    None
                                }
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
                error!("❌ Zypper search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via zypper");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("zypper", &["search", "--installed-only"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.starts_with("i | ") && line.contains(" | ") {
                            let parts: Vec<&str> = line.split(" | ").collect();
                            if parts.len() >= 2 {
                                let name = parts[1].trim();
                                if !name.is_empty() && name != "Name" {
                                    Some(name.to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Zypper list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "zypper".to_string(),
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
                .execute_package_command("zypper", &["info", package], config)
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

    fn needs_privilege(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        "zypper"
    }

    fn get_priority(&self) -> u8 {
        85 // High priority for openSUSE systems
    }
}
