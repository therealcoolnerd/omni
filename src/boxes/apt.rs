use crate::distro::PackageManager;
use crate::error_handling::OmniError;
use crate::runtime::RuntimeManager;
// use crate::secure_executor::{ExecutionConfig, SecureExecutor};
use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

/// Secure APT package manager wrapper
#[derive(Clone)]
pub struct AptManager {
    // executor: SecureExecutor, // Temporarily disabled
}

impl AptManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // executor: SecureExecutor::new()?, // Temporarily disabled
        })
    }

    pub fn is_available() -> bool {
        std::process::Command::new("apt")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    async fn install_internal(&self, package: &str) -> Result<()> {
        info!("Installing '{}' via apt", package);

        // First update package lists
        let update_config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };

        let update_result = self
            .executor
            .execute_package_command("apt", &["update"], update_config)
            .await;

        if let Err(e) = update_result {
            warn!("APT update failed, continuing with installation: {}", e);
        }

        // Install the package
        let install_config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(600),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("apt", &["install", "-y", package], install_config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ APT successfully installed '{}'", package);
            Ok(())
        } else {
            error!("❌ APT failed to install '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "apt".to_string(),
                reason: result.stderr,
            }
            .into())
        }
    }

    async fn remove_internal(&self, package: &str) -> Result<()> {
        info!("Removing '{}' via apt", package);

        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("apt", &["remove", "-y", package], config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ APT successfully removed '{}'", package);
            Ok(())
        } else {
            error!("❌ APT failed to remove '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "apt".to_string(),
                reason: format!("Remove failed: {}", result.stderr),
            }
            .into())
        }
    }

    pub async fn update_cache(&self) -> Result<()> {
        info!("Updating apt package cache");

        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("apt", &["update"], config)
            .await?;

        if result.exit_code == 0 {
            info!("✅ APT cache updated successfully");
            Ok(())
        } else {
            error!("❌ APT cache update failed: {}", result.stderr);
            Err(OmniError::InstallationFailed {
                package: "cache".to_string(),
                box_type: "apt".to_string(),
                reason: format!("Cache update failed: {}", result.stderr),
            }
            .into())
        }
    }

    async fn search_internal(&self, query: &str) -> Result<Vec<String>> {
        info!("Searching for '{}' via apt", query);

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(60),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("apt", &["search", query], config)
            .await?;

        if result.exit_code == 0 {
            let packages: Vec<String> = result
                .stdout
                .lines()
                .filter_map(|line| {
                    if line.starts_with("WARNING") || line.starts_with("NOTE") {
                        None
                    } else if let Some(package) = line.split('/').next() {
                        Some(package.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            info!("✅ Found {} packages matching '{}'", packages.len(), query);
            Ok(packages)
        } else {
            error!("❌ APT search failed: {}", result.stderr);
            Ok(vec![]) // Return empty list instead of error for search
        }
    }

    async fn get_package_info_internal(&self, package: &str) -> Result<String> {
        info!("Getting info for package '{}'", package);

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(30),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command("apt", &["show", package], config)
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
}

impl PackageManager for AptManager {
    fn install(&self, package: &str) -> Result<()> {
        RuntimeManager::block_on(self.install_internal(package))
    }

    fn remove(&self, package: &str) -> Result<()> {
        RuntimeManager::block_on(self.remove_internal(package))
    }

    fn update(&self, package: Option<&str>) -> Result<()> {
        let package_owned = package.map(|s| s.to_string());
        let apt_manager = self.clone();
        RuntimeManager::block_on(async move {
            if let Some(pkg) = package_owned {
                apt_manager.install_internal(&pkg).await
            } else {
                apt_manager.update_cache().await
            }
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        RuntimeManager::block_on(self.search_internal(query))
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let apt_manager = self.clone();
        RuntimeManager::block_on(async move {
            let packages = apt_manager.get_installed_packages().await?;
            Ok(packages.into_iter().map(|p| p.name).collect())
        })
    }

    fn get_info(&self, package: &str) -> Result<String> {
        RuntimeManager::block_on(self.get_package_info_internal(package))
    }

    fn needs_privilege(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        "apt"
    }

    fn get_priority(&self) -> u8 {
        90 // High priority for Debian/Ubuntu systems
    }
}

impl AptManager {
    // Public async methods
    pub async fn install_async(&self, package: &str) -> Result<()> {
        self.install_internal(package).await
    }

    pub async fn remove_async(&self, package: &str) -> Result<()> {
        self.remove_internal(package).await
    }

    pub async fn search_async(&self, query: &str) -> Result<Vec<String>> {
        self.search_internal(query).await
    }

    pub async fn get_package_info_async(&self, package: &str) -> Result<String> {
        self.get_package_info_internal(package).await
    }

    pub async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        info!("Getting installed packages via apt");

        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(60),
            ..ExecutionConfig::default()
        };

        let result = self
            .executor
            .execute_package_command(
                "dpkg-query",
                &["-W", "--showformat=${Package}\t${Version}\t${Status}\n"],
                config,
            )
            .await?;

        if result.exit_code == 0 {
            let packages: Vec<InstalledPackage> = result
                .stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 && parts[2].contains("installed") {
                        Some(InstalledPackage {
                            name: parts[0].to_string(),
                            version: parts[1].to_string(),
                            description: None,
                        })
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
                box_type: "apt".to_string(),
                reason: format!("List failed: {}", result.stderr),
            }
            .into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

// Legacy functions for backward compatibility
pub fn install_with_apt(package: &str) {
    match AptManager::new() {
        Ok(manager) => {
            if let Err(e) = manager.install(package) {
                error!("APT installation failed: {}", e);
            }
        }
        Err(e) => error!("Failed to create APT manager: {}", e),
    }
}

pub fn uninstall_with_apt(package: &str) {
    match AptManager::new() {
        Ok(manager) => {
            if let Err(e) = manager.remove(package) {
                error!("APT removal failed: {}", e);
            }
        }
        Err(e) => error!("Failed to create APT manager: {}", e),
    }
}
