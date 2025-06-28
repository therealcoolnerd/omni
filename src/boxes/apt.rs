use crate::secure_executor::{SecureExecutor, ExecutionConfig};
use crate::error_handling::OmniError;
use anyhow::Result;
use tracing::{info, error};
use std::time::Duration;

/// Secure APT package manager wrapper
pub struct AptManager {
    executor: SecureExecutor,
}

impl AptManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            executor: SecureExecutor::new()?,
        })
    }
    
    pub async fn install(&self, package: &str) -> Result<()> {
        info!("Installing '{}' via apt", package);
        
        let result = self.executor.secure_install(package, "apt", None).await?;
        
        if result.exit_code == 0 {
            info!("✅ APT successfully installed '{}'", package);
            Ok(())
        } else {
            error!("❌ APT failed to install '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "apt".to_string(),
                reason: result.stderr,
            }.into())
        }
    }
    
    pub async fn remove(&self, package: &str) -> Result<()> {
        info!("Removing '{}' via apt", package);
        
        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };
        
        let result = self.executor.execute_package_command(
            "apt",
            &["remove", "-y", package],
            config
        ).await?;
        
        if result.exit_code == 0 {
            info!("✅ APT successfully removed '{}'", package);
            Ok(())
        } else {
            error!("❌ APT failed to remove '{}': {}", package, result.stderr);
            Err(OmniError::InstallationFailed {
                package: package.to_string(),
                box_type: "apt".to_string(),
                reason: format!("Remove failed: {}", result.stderr),
            }.into())
        }
    }
    
    pub async fn update_cache(&self) -> Result<()> {
        info!("Updating apt package cache");
        
        let config = ExecutionConfig {
            requires_sudo: true,
            timeout: Duration::from_secs(300),
            ..ExecutionConfig::default()
        };
        
        let result = self.executor.execute_package_command(
            "apt",
            &["update"],
            config
        ).await?;
        
        if result.exit_code == 0 {
            info!("✅ APT cache updated successfully");
            Ok(())
        } else {
            error!("❌ APT cache update failed: {}", result.stderr);
            Err(OmniError::InstallationFailed {
                package: "cache".to_string(),
                box_type: "apt".to_string(),
                reason: format!("Cache update failed: {}", result.stderr),
            }.into())
        }
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        info!("Searching for '{}' via apt", query);
        
        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(60),
            ..ExecutionConfig::default()
        };
        
        let result = self.executor.execute_package_command(
            "apt",
            &["search", query],
            config
        ).await?;
        
        if result.exit_code == 0 {
            let packages: Vec<String> = result.stdout
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
    
    pub async fn get_package_info(&self, package: &str) -> Result<String> {
        info!("Getting info for package '{}'", package);
        
        let config = ExecutionConfig {
            requires_sudo: false,
            timeout: Duration::from_secs(30),
            ..ExecutionConfig::default()
        };
        
        let result = self.executor.execute_package_command(
            "apt",
            &["show", package],
            config
        ).await?;
        
        if result.exit_code == 0 {
            Ok(result.stdout)
        } else {
            Err(OmniError::PackageNotFound {
                package: package.to_string(),
            }.into())
        }
    }
}

// Legacy functions for backward compatibility
pub fn install_with_apt(package: &str) {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            match AptManager::new() {
                Ok(manager) => {
                    if let Err(e) = manager.install(package).await {
                        error!("APT installation failed: {}", e);
                    }
                }
                Err(e) => error!("Failed to create APT manager: {}", e),
            }
        });
}

pub fn uninstall_with_apt(package: &str) {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            match AptManager::new() {
                Ok(manager) => {
                    if let Err(e) = manager.remove(package).await {
                        error!("APT removal failed: {}", e);
                    }
                }
                Err(e) => error!("Failed to create APT manager: {}", e),
            }
        });
}
