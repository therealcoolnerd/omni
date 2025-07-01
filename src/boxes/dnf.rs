use crate::distro::PackageManager;
use crate::error_handling::{OmniError, RetryHandler, RetryConfig};
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use crate::types::InstalledPackage;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Secure DNF package manager wrapper
#[derive(Clone)]
pub struct DnfBox {
    executor: Arc<SecureExecutor>,
    retry_handler: RetryHandler,
}

impl DnfBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: Arc::new(SecureExecutor::new()?),
            retry_handler: RetryHandler::new(RetryConfig::new_network()),
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("dnf")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for DnfBox {
    fn install(&self, package: &str) -> Result<()> {
        let package = package.to_string();
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            info!("Installing '{}' via dnf", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            // First update package cache
            let update_config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            if let Err(e) = executor
                .execute_package_command("dnf", &["check-update"], update_config)
                .await
            {
                warn!("DNF cache check failed, continuing: {}", e);
            }

            let result = executor
                .execute_package_command("dnf", &["install", "-y", &package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ DNF successfully installed '{}'", package);
                Ok(())
            } else {
                error!("❌ DNF failed to install '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.clone(),
                    box_type: "dnf".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        let package = package.to_string();
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            info!("Removing '{}' via dnf", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("dnf", &["remove", "-y", &package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ DNF successfully removed '{}'", package);
                Ok(())
            } else {
                error!("❌ DNF failed to remove '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.clone(),
                    box_type: "dnf".to_string(),
                    reason: format!("Remove failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        let package_name = package.map(|s| s.to_string());
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            let mut args = vec!["upgrade".to_string(), "-y".to_string()];

            if let Some(ref pkg) = package_name {
                args.push(pkg.clone());
                info!("Upgrading '{}' via dnf", pkg);
            } else {
                info!("Upgrading all packages via dnf");
            }

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(1200), // 20 minutes for updates
                ..ExecutionConfig::default()
            };

            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let result = executor
                .execute_package_command("dnf", &args_refs, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ DNF update completed successfully");
                Ok(())
            } else {
                error!("❌ DNF update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package_name.unwrap_or_else(|| "all".to_string()),
                    box_type: "dnf".to_string(),
                    reason: format!("Update failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        let query = query.to_string();
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            info!("Searching for '{}' via dnf", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("dnf", &["search", &query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.contains(".")
                            && !line.starts_with("=")
                            && !line.starts_with("Last metadata")
                        {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if !parts.is_empty() {
                                Some(parts[0].split('.').next().unwrap_or(parts[0]).to_string())
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
                error!("❌ DNF search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            info!("Listing installed packages via dnf");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("dnf", &["list", "installed"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .skip(1) // Skip header line
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 && parts[0].contains(".") {
                            Some(parts[0].split('.').next().unwrap_or(parts[0]).to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ DNF list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "dnf".to_string(),
                    reason: format!("List failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        let package = package.to_string();
        let executor = Arc::clone(&self.executor);

        RuntimeManager::block_on(async move {
            info!("Getting info for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("dnf", &["info", &package], config)
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                Err(OmniError::PackageNotFound {
                    package: package.clone(),
                }
                .into())
            }
        })
    }

    fn needs_privilege(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        "dnf"
    }

    fn get_priority(&self) -> u8 {
        85 // High priority for Red Hat systems
    }
}

impl DnfBox {
    // Public async methods for better integration
    pub async fn install_async(&self, package: &str) -> Result<()> {
        info!("Installing '{}' via dnf", package);

        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(600),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["install", "-y", package], config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ DNF successfully installed '{}'", package);
            Ok(())
        } else {
            error!("❌ DNF failed to install '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "dnf".to_string(),
                reason: result.stderr,
            }.into())
        }
    }

    pub async fn remove_async(&self, package: &str) -> Result<()> {
        info!("Removing '{}' via dnf", package);

        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["remove", "-y", package], config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ DNF successfully removed '{}'", package);
            Ok(())
        } else {
            error!("❌ DNF failed to remove '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "dnf".to_string(),
                reason: format!("Remove failed: {}", result.stderr),
            }.into())
        }
    }

    pub async fn search_async(&self, query: &str) -> Result<Vec<String>> {
        info!("Searching for '{}' via dnf", query);

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(60),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["search", query], config)
            .await?;

        if result.exit_code == 0 {
            let packages: Vec<String> = result
                .stdout
                .lines()
                .filter_map(|line| {
                    if line.contains(".")
                        && !line.starts_with("=")
                        && !line.starts_with("Last metadata")
                    {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            Some(parts[0].split('.').next().unwrap_or(parts[0]).to_string())
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
            error!("❌ DNF search failed: {}", result.stderr);
            Ok(vec![]) // Return empty list instead of error for search
        }
    }

    pub async fn get_package_info_async(&self, package: &str) -> Result<String> {
        info!("Getting info for package '{}'", package);

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(30),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["info", package], config)
            .await?;

        if result.exit_code == 0 {
            Ok(result.stdout)
        } else {
            Err(OmniError::PackageNotFound {
                package: package.to_string(),
            }.into())
        }
    }

    pub async fn update_cache(&self) -> Result<()> {
        info!("Updating dnf package cache");

        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["makecache"], config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ DNF cache updated successfully");
            Ok(())
        } else {
            error!("❌ DNF cache update failed: {}", result.stderr);
            Err(OmniError::InstallationFailed {
                package: "cache".to_string(),
                box_type: "dnf".to_string(),
                reason: format!("Cache update failed: {}", result.stderr),
            }.into())
        }
    }

    pub async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        info!("Getting installed packages via dnf");

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(60),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("dnf", &["list", "installed"], config)
            .await?;

        if result.exit_code == 0 {
            let packages: Vec<InstalledPackage> = result
                .stdout
                .lines()
                .skip(1) // Skip header line
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[0].contains(".") {
                        Some(InstalledPackage::with_description(
                            parts[0].split('.').next().unwrap_or(parts[0]).to_string(),
                            parts[1].to_string(),
                            None,
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            info!("✅ Found {} installed packages", packages.len());
            Ok(packages)
        } else {
            error!("❌ Failed to get installed packages: {}", result.stderr);
            Err(OmniError::InstallationFailed {
                package: "list".to_string(),
                box_type: "dnf".to_string(),
                reason: format!("List failed: {}", result.stderr),
            }.into())
        }
    }
}


// Legacy functions for backward compatibility
pub fn install_with_dnf(package: &str) {
    match DnfBox::new() {
        Ok(manager) => {
            if let Err(e) = manager.install(package) {
                error!("DNF installation failed: {}", e);
            }
        }
        Err(e) => error!("Failed to create DNF manager: {}", e),
    }
}

pub fn uninstall_with_dnf(package: &str) {
    match DnfBox::new() {
        Ok(manager) => {
            if let Err(e) = manager.remove(package) {
                error!("DNF removal failed: {}", e);
            }
        }
        Err(e) => error!("Failed to create DNF manager: {}", e),
    }
}
