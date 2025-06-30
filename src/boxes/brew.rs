use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info};

pub struct BrewBox {
    executor: SecureExecutor,
}

impl BrewBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for BrewBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via brew", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("brew", &["install", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Brew successfully installed '{}'", package);
                Ok(())
            } else {
                error!("❌ Brew failed to install '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "brew".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via brew", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("brew", &["uninstall", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Brew successfully removed '{}'", package);
                Ok(())
            } else {
                error!("❌ Brew failed to remove '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "brew".to_string(),
                    reason: format!("Remove failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            // First update brew itself
            info!("Updating brew repositories");
            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let _ = self
                .executor
                .execute_package_command("brew", &["update"], config.clone())
                .await;

            // Then upgrade packages
            let mut args = vec!["upgrade"];

            if let Some(pkg) = package {
                args.push(pkg);
                info!("Upgrading '{}' via brew", pkg);
            } else {
                info!("Upgrading all packages via brew");
            }

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(1200), // 20 minutes for updates
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("brew", &args, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Brew update completed successfully");
                Ok(())
            } else {
                error!("❌ Brew update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.unwrap_or("all").to_string(),
                    box_type: "brew".to_string(),
                    reason: format!("Update failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via brew", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("brew", &["search", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
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

                info!("✅ Found {} packages matching '{}'", packages.len(), query);
                Ok(packages)
            } else {
                error!("❌ Brew search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via brew");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("brew", &["list"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Brew list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "brew".to_string(),
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
                .execute_package_command("brew", &["info", package], config)
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
        false // Homebrew doesn't require admin privileges
    }

    fn get_name(&self) -> &'static str {
        "brew"
    }

    fn get_priority(&self) -> u8 {
        80 // High priority for macOS systems
    }
}
