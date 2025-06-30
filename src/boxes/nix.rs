use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info};

/// Secure Nix package manager wrapper
pub struct NixBox {
    executor: SecureExecutor,
}

impl NixBox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("nix")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl PackageManager for NixBox {
    fn install(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Installing '{}' via nix", package);

            let config = ExecutionConfig {
                requires_sudo: false, // Nix doesn't require sudo for user profile
                timeout: Duration::from_secs(600),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command(
                    "nix-env",
                    &["-iA", &format!("nixpkgs.{}", package)],
                    config,
                )
                .await?;

            if result.exit_code == 0 {
                info!("✅ Nix successfully installed '{}'", package);
                Ok(())
            } else {
                error!("❌ Nix failed to install '{}': {}", package, result.stderr);
                // Try alternative syntax
                let config2 = ExecutionConfig {
                    requires_sudo: false,
                    timeout: Duration::from_secs(600),
                    ..ExecutionConfig::default()
                };

                let result2 = self
                    .executor
                    .execute_package_command("nix-env", &["-i", package], config2)
                    .await?;

                if result2.exit_code == 0 {
                    info!(
                        "✅ Nix successfully installed '{}' (fallback method)",
                        package
                    );
                    Ok(())
                } else {
                    error!(
                        "❌ Nix failed to install '{}' with both methods: {}",
                        package, result2.stderr
                    );
                    Err(OmniError::InstallationFailed {
                        package: package.to_string(),
                        box_type: "nix".to_string(),
                        reason: format!("Primary: {} Fallback: {}", result.stderr, result2.stderr),
                    }
                    .into())
                }
            }
        })
    }

    fn remove(&self, package: &str) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Removing '{}' via nix", package);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(300),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("nix-env", &["-e", package], config)
                .await?;

            if result.exit_code == 0 {
                info!("✅ Nix successfully removed '{}'", package);
                Ok(())
            } else {
                error!("❌ Nix failed to remove '{}': {}", package, result.stderr);
                Err(OmniError::InstallationFailed {
                    package: package.to_string(),
                    box_type: "nix".to_string(),
                    reason: format!("Remove failed: {}", result.stderr),
                }
                .into())
            }
        })
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        tokio::runtime::Runtime::new()?.block_on(async {
            if let Some(pkg) = package {
                // Update specific package by reinstalling
                info!("Upgrading '{}' via nix", pkg);
                self.install(pkg)
            } else {
                // Full system update
                info!("Updating nix channels and upgrading all packages");

                // First update channels
                let config = ExecutionConfig {
                    requires_sudo: false,
                    timeout: Duration::from_secs(300),
                    ..ExecutionConfig::default()
                };

                let _update_result = self
                    .executor
                    .execute_package_command("nix-channel", &["--update"], config.clone())
                    .await;

                // Then upgrade all packages
                let config = ExecutionConfig {
                    requires_sudo: false,
                    timeout: Duration::from_secs(1200), // 20 minutes for updates
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("nix-env", &["-u"], config)
                    .await?;

                if result.exit_code == 0 {
                    info!("✅ Nix system update completed successfully");
                    Ok(())
                } else {
                    error!("❌ Nix system update failed: {}", result.stderr);
                    Err(OmniError::InstallationFailed {
                        package: "all".to_string(),
                        box_type: "nix".to_string(),
                        reason: format!("Update failed: {}", result.stderr),
                    }
                    .into())
                }
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Searching for '{}' via nix", query);

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("nix", &["search", "nixpkgs", query, "--json"], config)
                .await?;

            if result.exit_code == 0 {
                // Parse JSON output to extract package names
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        if line.contains("\"pname\"") {
                            // Extract package name from JSON
                            if let Some(start) = line.find("\"pname\": \"") {
                                let start = start + 10;
                                if let Some(end) = line[start..].find("\"") {
                                    return Some(line[start..start + end].to_string());
                                }
                            }
                        }
                        None
                    })
                    .collect();

                // Fallback to simpler search if JSON parsing fails
                if packages.is_empty() {
                    let config = ExecutionConfig {
                        requires_sudo: false,
                        timeout: Duration::from_secs(60),
                        ..ExecutionConfig::default()
                    };

                    let result = self
                        .executor
                        .execute_package_command("nix-env", &["-qaP", query], config)
                        .await?;

                    let packages: Vec<String> = result
                        .stdout
                        .lines()
                        .filter_map(|line| {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let attr_path = parts[0];
                                if let Some(pkg_name) = attr_path.split('.').last() {
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
                    info!("✅ Found {} packages matching '{}'", packages.len(), query);
                    Ok(packages)
                }
            } else {
                error!("❌ Nix search failed: {}", result.stderr);
                Ok(vec![]) // Return empty list instead of error for search
            }
        })
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        tokio::runtime::Runtime::new()?.block_on(async {
            info!("Listing installed packages via nix");

            let config = ExecutionConfig {
                requires_sudo: false,
                timeout: Duration::from_secs(60),
                ..ExecutionConfig::default()
            };

            let result = self
                .executor
                .execute_package_command("nix-env", &["-q"], config)
                .await?;

            if result.exit_code == 0 {
                let packages: Vec<String> = result
                    .stdout
                    .lines()
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            // Remove version information
                            let name = trimmed.split('-').next().unwrap_or(trimmed);
                            Some(name.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                info!("✅ Found {} installed packages", packages.len());
                Ok(packages)
            } else {
                error!("❌ Nix list failed: {}", result.stderr);
                Err(OmniError::InstallationFailed {
                    package: "list".to_string(),
                    box_type: "nix".to_string(),
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
                .execute_package_command(
                    "nix",
                    &["show-derivation", &format!("nixpkgs#{}", package)],
                    config,
                )
                .await?;

            if result.exit_code == 0 {
                Ok(result.stdout)
            } else {
                // Fallback to simpler info command
                let config = ExecutionConfig {
                    requires_sudo: false,
                    timeout: Duration::from_secs(30),
                    ..ExecutionConfig::default()
                };

                let result = self
                    .executor
                    .execute_package_command("nix-env", &["-qa", "--description", package], config)
                    .await?;

                if result.exit_code == 0 {
                    Ok(result.stdout)
                } else {
                    Err(OmniError::PackageNotFound {
                        package: package.to_string(),
                    }
                    .into())
                }
            }
        })
    }

    fn needs_privilege(&self) -> bool {
        false // Nix works in user space
    }

    fn get_name(&self) -> &'static str {
        "nix"
    }

    fn get_priority(&self) -> u8 {
        75 // Medium-high priority, works on multiple platforms
    }
}
