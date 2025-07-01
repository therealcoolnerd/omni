use crate::config::OmniConfig;
use crate::unified_manager::UnifiedPackageManager;
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::snapshot::SnapshotManager;
use crate::manifest::OmniManifest;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error};
use std::collections::HashMap;

pub struct SecureOmniBrainV2 {
    config: OmniConfig,
    unified_manager: UnifiedPackageManager,
    db: Option<Database>,
    snapshot_manager: Option<SnapshotManager>,
}

impl SecureOmniBrainV2 {
    pub fn new() -> Result<Self> {
        let config = OmniConfig::load()?;
        let unified_manager = UnifiedPackageManager::new(config.clone())?;
        
        Ok(Self {
            config,
            unified_manager,
            db: None,
            snapshot_manager: None,
        })
    }
    
    pub fn new_with_config(config: OmniConfig) -> Result<Self> {
        let unified_manager = UnifiedPackageManager::new(config.clone())?;
        
        Ok(Self {
            config,
            unified_manager,
            db: None,
            snapshot_manager: None,
        })
    }
    
    async fn ensure_initialized(&mut self) -> Result<()> {
        if self.db.is_none() {
            self.db = Some(Database::new().await?);
        }
        
        if self.snapshot_manager.is_none() {
            self.snapshot_manager = Some(SnapshotManager::new().await?);
        }
        
        Ok(())
    }
    
    pub async fn install(&mut self, package: &str) -> Result<()> {
        self.ensure_initialized().await?;
        
        info!("Installing package: {}", package);
        
        // Create snapshot before installation if enabled
        if let Some(snapshot_manager) = &self.snapshot_manager {
            if self.config.general.confirm_installs {
                info!("Creating pre-installation snapshot");
                snapshot_manager.create_snapshot(&format!("pre-install-{}", package), Some("Pre-installation system snapshot")).await?;
            }
        }
        
        match self.unified_manager.install(package) {
            Ok(box_type) => {
                info!("✅ Successfully installed '{}' with {}", package, box_type);
                
                // Record the successful installation
                if let Some(db) = &self.db {
                    let install_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: package.to_string(),
                        box_type: box_type.clone(),
                        version: Some("unknown".to_string()), // TODO: Get actual version
                        source_url: None,
                        install_path: None,
                        installed_at: Utc::now(),
                        status: InstallStatus::Success,
                        metadata: None,
                    };
                    
                    let _ = db.record_install(&install_record).await;
                }
                
                // Create post-installation snapshot if enabled
                if let Some(snapshot_manager) = &self.snapshot_manager {
                    if self.config.general.confirm_installs {
                        info!("Creating post-installation snapshot");
                        snapshot_manager.create_snapshot(&format!("post-install-{}", package), Some("Post-installation system snapshot")).await?;
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to install '{}': {}", package, e);
                
                // Record the failed installation
                if let Some(db) = &self.db {
                    let install_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: package.to_string(),
                        box_type: "failed".to_string(),
                        version: None,
                        source_url: None,
                        install_path: None,
                        installed_at: Utc::now(),
                        status: InstallStatus::Failed,
                        metadata: Some(format!("Error: {}", e)),
                    };
                    
                    let _ = db.record_install(&install_record).await;
                }
                
                Err(e)
            }
        }
    }
    
    pub async fn install_with_box(&mut self, package: &str, box_type: &str) -> Result<()> {
        self.ensure_initialized().await?;
        
        info!("Installing package '{}' with specific box: {}", package, box_type);
        
        // Check if the box is enabled in configuration
        if !self.config.is_box_enabled(box_type) {
            return Err(anyhow!("Package manager '{}' is disabled in configuration", box_type));
        }
        
        match self.unified_manager.install_with_box(package, Some(box_type)) {
            Ok(used_box) => {
                info!("✅ Successfully installed '{}' with {}", package, used_box);
                
                // Record the successful installation
                if let Some(db) = &self.db {
                    let install_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: package.to_string(),
                        box_type: used_box,
                        version: Some("unknown".to_string()),
                        source_url: None,
                        install_path: None,
                        installed_at: Utc::now(),
                        status: InstallStatus::Success,
                        metadata: None,
                    };
                    
                    let _ = db.record_install(&install_record).await;
                }
                
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to install '{}' with {}: {}", package, box_type, e);
                Err(e)
            }
        }
    }
    
    pub async fn remove(&mut self, package: &str) -> Result<()> {
        self.ensure_initialized().await?;
        
        info!("Removing package: {}", package);
        
        match self.unified_manager.remove(package) {
            Ok(box_type) => {
                info!("✅ Successfully removed '{}' with {}", package, box_type);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to remove '{}': {}", package, e);
                Err(e)
            }
        }
    }
    
    pub async fn update(&mut self, package: Option<&str>) -> Result<()> {
        self.ensure_initialized().await?;
        
        if let Some(pkg) = package {
            info!("Updating package: {}", pkg);
        } else {
            info!("Updating all packages");
        }
        
        match self.unified_manager.update(package) {
            Ok(()) => {
                info!("✅ Successfully updated packages");
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update packages: {}", e);
                Err(e)
            }
        }
    }
    
    pub async fn search(&mut self, query: &str) -> Result<HashMap<String, Vec<String>>> {
        self.ensure_initialized().await?;
        
        info!("Searching for: {}", query);
        
        let results = self.unified_manager.search(query)?;
        
        for (box_name, packages) in &results {
            info!("Found {} packages in {}", packages.len(), box_name);
        }
        
        Ok(results)
    }
    
    pub async fn list_installed(&mut self) -> Result<HashMap<String, Vec<String>>> {
        self.ensure_initialized().await?;
        
        info!("Listing installed packages");
        
        let results = self.unified_manager.list_installed()?;
        
        for (box_name, packages) in &results {
            info!("Found {} installed packages in {}", packages.len(), box_name);
        }
        
        Ok(results)
    }
    
    pub async fn get_package_info(&mut self, package: &str, box_type: &str) -> Result<String> {
        self.ensure_initialized().await?;
        
        info!("Getting info for package '{}' from {}", package, box_type);
        
        self.unified_manager.get_info(package, box_type)
    }
    
    pub async fn install_from_manifest(&mut self, manifest: OmniManifest) -> Result<()> {
        self.ensure_initialized().await?;
        
        info!("Installing from manifest: {}", manifest.project);
        
        if let Some(desc) = &manifest.description {
            info!("Description: {}", desc);
        }
        
        for app in &manifest.apps {
            info!("Installing {} via {} box", app.name, app.box_type);
            
            if let Some(source) = &app.source {
                info!("Source: {}", source);
            }
            
            match self.install_with_box(&app.name, &app.box_type).await {
                Ok(()) => {
                    info!("✅ Successfully installed {} from manifest", app.name);
                }
                Err(e) => {
                    error!("❌ Failed to install {} from manifest: {}", app.name, e);
                    if !self.config.general.fallback_enabled {
                        return Err(e);
                    }
                    // Continue with other packages if fallback is enabled
                }
            }
        }
        
        info!("✅ Manifest installation completed");
        Ok(())
    }
    
    pub fn get_available_managers(&self) -> Vec<String> {
        self.unified_manager.get_available_managers()
    }
    
    pub fn get_config(&self) -> &OmniConfig {
        &self.config
    }
    
    pub async fn reload_config(&mut self) -> Result<()> {
        info!("Reloading configuration");
        self.config = OmniConfig::load()?;
        self.unified_manager.reload_config(self.config.clone())?;
        Ok(())
    }
}