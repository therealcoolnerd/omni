use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

/// Secure Emerge package manager wrapper for Gentoo
pub struct EmergeBox {
    executor: SecureExecutor,
}

impl EmergeBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("emerge")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for EmergeBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via emerge", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(3600), // 1 hour for compilation
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("emerge", &["--ask", "n", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Emerge successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Emerge failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "emerge".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via emerge", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("emerge", &["--unmerge", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Emerge successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Emerge failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "emerge".to_string(),
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
                info!("Upgrading '{}' via emerge", pkg);
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(3600), // 1 hour for compilation
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("emerge", &["--update", "--ask", "n", pkg], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Emerge upgrade completed successfully");
                    Ok(())
                } else {
                    error!("❌ Emerge upgrade failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: pkg.to_string(),
                        box_type: "emerge".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            } else {
                // Full system update
                info!("Updating all packages via emerge");
                let config = ExecutionConfig {
                    requires_sudo: true,
                    timeout: Duration::from_secs(7200), // 2 hours for world update
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command(
                        "emerge",
                        &["--update", "--deep", "--newuse", "--ask", "n", "@world"],
                        config,
                    )
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Emerge world update completed successfully");
                    Ok(())
                } else {
                    error!("❌ Emerge world update failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: "world".to_string(),
                        box_type: "emerge".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via emerge", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("emerge", &["--search", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.starts_with("*  ") {
                            // Package name line
                            let name = line.trim_start_matches("*  ");
                            if let Some(category_package) = name.split_whitespace().next() {
                                if category_package.contains("/") {
                                    let pkg_name = category_package
                                        .split('/')
                                        .nth(1)
                                        .unwrap_or(category_package);
                                    Some(pkg_name.to_string())
                                } else {
                                    Some(category_package.to_string())
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
                error!("❌ Emerge search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via emerge");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            // Use qlist from portage-utils which is faster
            let result = self
                .executor
                .execute_package_command("qlist", &["-I"], config.clone())
                .await;

            let result = match result {
                Ok(r) => r,
                Err(_) => {
                    // Fallback to emerge if qlist is not available
                    self.executor
                        .execute_package_command("emerge", &["--list"], config)
                        .await?
                }
            };

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.contains("/") {
                            let pkg_name = trimmed.split('/').nth(1).unwrap_or(trimmed);
                            Some(pkg_name.split('-').next().unwrap_or(pkg_name).to_string())
                        } else if !trimmed.is_empty() {
                            Some(trimmed.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Emerge list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "emerge".to_string(),
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
                .execute_package_command("emerge", &["--info", package], config)
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
        let executor = self.executor.clone();
        
        RuntimeManager::block_on(async move {
            info!("Getting installed version for package '{}'", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(30),
                ..ExecutionConfig::default()
            };

            let result = executor
                .execute_package_command("qlist", &["-I", "-v", &package], config)
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse qlist output: category/package-version
                for line in result.stdout.lines() {
                    if line.contains(&package) {
                        // Extract version from category/package-version format
                        if let Some(last_dash) = line.rfind('-') {
                            let version = line[last_dash + 1..].to_string();
                            if !version.is_empty() {
                                info!("✅ Found installed version '{}' for package '{}'", version, package);
                                return Ok(Some(version));
                            }
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
        true
    }

    fn get_name(&self) -> &'static str {
        "emerge"
    }

    fn get_priority(&self) -> u8 {
        80 // High priority for Gentoo systems
    }
}
