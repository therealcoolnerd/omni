use crate::distro::PackageManager;
use crate::error_handling::{OmniError, RetryConfig, RetryHandler};
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct WingetBox {
    executor: SecureExecutor,
    retry_handler: RetryHandler,
}

impl WingetBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
            retry_handler: RetryHandler::new(RetryConfig::new_network()),
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("winget")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for WingetBox {
    fn install(&self, package: &str) -> Result<()> {
        let package = package.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Installing '{}' via winget", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command(
                    "winget",
                    &[
                        "install",
                        &package,
                        "--accept-package-agreements",
                        "--accept-source-agreements",
                    ],
                    config,
                )
                .await?;

            if result.exit_code == 0 {
                info!("✅ Winget successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Winget failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "winget".to_string(),
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
            info!("Removing '{}' via winget", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("winget", &["uninstall", &package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Winget successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Winget failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "winget".to_string(),
                    reason: format!("Remove failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        let package_owned = package.map(|s| s.to_string());
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            let mut args = vec!["upgrade"];

            if let Some(ref pkg) = package_owned {
                args.push(pkg);
                info!("Updating '{}' via winget", pkg);
            } else {
                args.push("--all");
                info!("Updating all packages via winget");
            }
            args.extend(&["--accept-package-agreements", "--accept-source-agreements"]);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(1200), // 20 minutes for updates
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("winget", &args, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Winget update completed successfully");
                Ok(())
            } else {
                error!("❌ Winget update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package_owned.unwrap_or_else(|| "all".to_string()),
                    box_type: "winget".to_string(),
                    reason: format!("Update failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        let query = query.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Searching for '{}' via winget", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("winget", &["search", &query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
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

                info!("✅ Found {} packages matching '{}'", packages.len(), query);
                Ok(packages)
            } else {
                error!("❌ Winget search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Listing installed packages via winget");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("winget", &["list"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
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
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Winget list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "winget".to_string(),
                    reason: format!("List failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        let package = package.to_string();
        let executor = self.executor.clone();
        RuntimeManager::block_on(async move {
            info!("Getting info for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("winget", &["show", &package], config)
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
                .execute_package_command("winget", &["list", "--exact", "--id", &package], config)
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse winget list output - look for the package line
                for line in result.stdout.lines() {
                    if line.contains(&package) && !line.starts_with("Name") && !line.starts_with("-") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            // Typically format: Name Id Version [Available] [Source]
                            let version = parts[2].to_string();
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
        false // winget typically doesn't require admin privileges
    }

    fn get_name(&self) -> &'static str {
        "winget"
    }

    fn get_priority(&self) -> u8 {
        80 // High priority for Windows systems
    }
}
