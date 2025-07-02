use crate::distro::PackageManager;
use crate::error_handling::{OmniError, RetryConfig, RetryHandler};
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct SnapBox {
    executor: SecureExecutor,
    retry_handler: RetryHandler,
}

impl SnapBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
            retry_handler: RetryHandler::new(RetryConfig::new_network()),
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("snap")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for SnapBox {
    fn install(&self, package: &str) -> Result<()> {
        let package = package.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Installing '{}' via snap", package);

            let config = ExecutionConfig {
                requires_sudo: true, // Snap typically requires sudo for installation
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["install", &package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Snap successfully installed '{}'", package);
                Ok(())
            } else {
                error!("❌ Snap failed to install '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        let package = package.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Removing '{}' via snap", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["remove", &package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Snap successfully removed '{}'", package);
                Ok(())
            } else {
                error!("❌ Snap failed to remove '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        let package_owned = package.map(|s| s.to_string());
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            let mut args = vec!["refresh"];

            if let Some(ref pkg) = package_owned {
                args.push(pkg);
                info!("Updating '{}' via snap", pkg);
            } else {
                info!("Updating all packages via snap");
            }

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(1800), // 30 minutes for updates
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &args, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Snap update completed successfully");
                Ok(())
            } else {
                error!("❌ Snap update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package_owned.unwrap_or_else(|| "all".to_string()),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        let query = query.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Searching for '{}' via snap", query);

            let config = ExecutionConfig {
                requires_sudo: false, // Search doesn't require sudo
                timeout: Duration::from_secs(120),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["find", &query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .skip(1) // Skip header line
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.first().map(|s| s.to_string())
                    })
                    .collect();
                Ok(packages)
            } else {
                error!("❌ Snap search failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: query.to_string(),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Listing installed packages via snap");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["list"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .skip(1) // Skip header line
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.first().map(|s| s.to_string())
                    })
                    .collect();
                Ok(packages)
            } else {
                error!("❌ Snap list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        let package = package.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Getting info for '{}' via snap", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["info", &package], config)
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                error!("❌ Snap info failed for '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "snap".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        let package = package.to_string();
        let executor = self.executor.clone();
        
        RuntimeManager::block_on(async move {
            info!("Getting installed version for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("snap", &["list", &package], config)
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse snap list output: Name Version Rev Tracking Publisher Notes
                for line in result.stdout.lines() {
                    if line.starts_with(&package) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 && parts[0] == package {
                            let version = parts[1].to_string();
                            info!("✅ Found installed version '{}' for package '{}'", version, package);
                            return Ok(Some(version));
                        }
                    }
                }
                info!("ℹ️ Package '{}' output format unexpected: {}", package, result.stdout.trim());
                Ok(None)
            } else {
                info!("ℹ️ Package '{}' is not installed", package);
                Ok(None)
            }
        })
    }

    fn needs_privilege(&self) -> bool {
        true // Snap requires sudo for install/remove operations
    }

    fn get_name(&self) -> &'static str {
        "snap"
    }

    fn get_priority(&self) -> u8 {
        60 // Medium priority for Ubuntu systems
    }
}

// Backward compatibility functions
pub fn install_with_snap(app: &str) -> anyhow::Result<()> {
    let snap_box = SnapBox::new()?;
    snap_box.install(app)
}

pub fn search_snap(query: &str) -> anyhow::Result<Vec<String>> {
    let snap_box = SnapBox::new()?;
    snap_box.search(query)
}

pub fn get_snap_info(app: &str) -> anyhow::Result<String> {
    let snap_box = SnapBox::new()?;
    snap_box.get_info(app)
}

pub fn remove_snap(app: &str) -> anyhow::Result<()> {
    let snap_box = SnapBox::new()?;
    snap_box.remove(app)
}

pub fn update_snap(app: &str) -> anyhow::Result<()> {
    let snap_box = SnapBox::new()?;
    snap_box.update(Some(app))
}
