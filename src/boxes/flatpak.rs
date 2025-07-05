use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct FlatpakBox {
    executor: SecureExecutor,
}

impl FlatpakBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("flatpak")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for FlatpakBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via flatpak", package);

            let config = ExecutionConfig {
                requires_sudo: false, // Flatpak typically doesn't require sudo for user installations
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &["install", "-y", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Flatpak successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Flatpak failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "flatpak".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via flatpak", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &["uninstall", "-y", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Flatpak successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Flatpak failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "flatpak".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let mut args = vec!["update", "-y"];

            if let Some(pkg) = package {
                args.push(pkg);
                info!("Updating '{}' via flatpak", pkg);
            } else {
                info!("Updating all packages via flatpak");
            }

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(1800), // 30 minutes for updates
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &args, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Flatpak update completed successfully");
                Ok(())
            } else {
                error!("❌ Flatpak update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.unwrap_or("all").to_string(),
                    box_type: "flatpak".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via flatpak", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(120),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &["search", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .skip(1) // Skip header line
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        // Flatpak search output format: Name Description Application ID Version Branch Remotes
                        if parts.len() >= 3 {
                            Some(parts[2].to_string()) // Application ID
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(packages)
            } else {
                error!("❌ Flatpak search failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: query.to_string(),
                    box_type: "flatpak".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via flatpak");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &["list", "--app"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .skip(1) // Skip header line
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split('\t').collect();
                        // Flatpak list output format is tab-separated: Name Application ID Version Branch Installation
                        if parts.len() >= 2 {
                            Some(parts[1].to_string()) // Application ID
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(packages)
            } else {
                error!("❌ Flatpak list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "flatpak".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Getting info for '{}' via flatpak", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("flatpak", &["info", package], config)
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                error!(
                    "❌ Flatpak info failed for '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "flatpak".to_string(),
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
                .execute_package_command(
                    "flatpak",
                    &["list", "--app", "--columns=name,version", &package],
                    config,
                )
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse flatpak list output
                for line in result.stdout.lines() {
                    if line.contains(&package) {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 2 {
                            let version = parts[1].trim().to_string();
                            if !version.is_empty() {
                                info!(
                                    "✅ Found installed version '{}' for package '{}'",
                                    version, package
                                );
                                return Ok(Some(version));
                            }
                        }
                    }
                }
                info!(
                    "ℹ️ Package '{}' output format unexpected: {}",
                    package,
                    result.stdout.trim()
                );
                Ok(None)
            } else {
                info!("ℹ️ Package '{}' is not installed", package);
                Ok(None)
            }
        })
    }

    fn needs_privilege(&self) -> bool {
        false // Flatpak typically doesn't require sudo for user installations
    }

    fn get_name(&self) -> &'static str {
        "flatpak"
    }

    fn get_priority(&self) -> u8 {
        50 // Medium priority for Linux systems with Flatpak
    }
}

// Backward compatibility functions
pub fn install_with_flatpak(package: &str) {
    if let Ok(flatpak_box) = FlatpakBox::new() {
        let _ = flatpak_box.install(package);
    }
}

pub fn uninstall_with_flatpak(package: &str) {
    if let Ok(flatpak_box) = FlatpakBox::new() {
        let _ = flatpak_box.remove(package);
    }
}
