use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::runtime::RuntimeManager;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct ChocolateyBox {
    executor: SecureExecutor,
}

impl ChocolateyBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("choco")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for ChocolateyBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via chocolatey", package);

            let config = ExecutionConfig {
                requires_sudo: true, // Chocolatey requires admin
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &["install", package, "-y"], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Chocolatey successfully installed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Chocolatey failed to install '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "chocolatey".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via chocolatey", package);

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &["uninstall", package, "-y"], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Chocolatey successfully removed '{}'", package);
                Ok(())
            } else {
                error!(
                    "❌ Chocolatey failed to remove '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "chocolatey".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let mut args = vec!["upgrade"];

            if let Some(pkg) = package {
                args.push(pkg);
                info!("Updating '{}' via chocolatey", pkg);
            } else {
                args.push("all");
                info!("Updating all packages via chocolatey");
            }
            args.push("-y");

            let config = ExecutionConfig {
                requires_sudo: true,
                timeout: Duration::from_secs(1800), // 30 minutes for updates
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &args, config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Chocolatey update completed successfully");
                Ok(())
            } else {
                error!("❌ Chocolatey update failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.unwrap_or("all").to_string(),
                    box_type: "chocolatey".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via chocolatey", query);

            let config = ExecutionConfig {
                requires_sudo: false, // Search doesn't require admin
                timeout: Duration::from_secs(120),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &["search", query], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.contains(" | ") && !line.starts_with("Chocolatey") {
                            let parts: Vec<&str> = line.split(" | ").collect();
                            if !parts.is_empty() {
                                Some(parts[0].trim().to_string())
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
                error!("❌ Chocolatey search failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: query.to_string(),
                    box_type: "chocolatey".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via chocolatey");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &["list", "--local-only"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.contains(" ")
                            && !line.starts_with("Chocolatey")
                            && !line.contains("packages installed")
                        {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if !parts.is_empty() {
                                Some(parts[0].to_string())
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
                error!("❌ Chocolatey list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "chocolatey".to_string(),
                    reason: result.stderr,
                }
                .into())
            }
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Getting info for '{}' via chocolatey", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("choco", &["info", package], config)
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                error!(
                    "❌ Chocolatey info failed for '{}': {}",
                    package, result.stderr
                );
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "chocolatey".to_string(),
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
                .execute_package_command("choco", &["list", "--local-only", "--exact", &package], config)
                .await?;

            if result.exit_code == 0 && !result.stdout.trim().is_empty() {
                // Parse chocolatey list output: packagename version
                for line in result.stdout.lines() {
                    if line.starts_with(&package) && line.contains(' ') {
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
        true // Chocolatey typically requires admin privileges
    }

    fn get_name(&self) -> &'static str {
        "chocolatey"
    }

    fn get_priority(&self) -> u8 {
        70 // Medium-high priority for Windows systems
    }
}
