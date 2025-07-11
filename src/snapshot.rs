use crate::boxes::appimage;
use crate::boxes::apt::AptManager;
use crate::boxes::dnf::DnfBox;
use crate::boxes::flatpak::FlatpakBox;
use crate::boxes::pacman::PacmanBox;
use crate::boxes::snap::SnapBox;
use crate::database::{Database, InstallRecord, InstallStatus, Snapshot};
use crate::distro::{self, PackageManager};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct SnapshotManager {
    db: Database,
}

impl SnapshotManager {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        Ok(Self { db })
    }

    pub async fn create_snapshot(&self, name: &str, description: Option<&str>) -> Result<String> {
        info!("Creating snapshot: {}", name);

        let snapshot_id = self.db.create_snapshot(name, description).await?;

        info!(
            "✅ Successfully created snapshot '{}' with ID: {}",
            name, snapshot_id
        );
        Ok(snapshot_id)
    }

    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        self.db.list_snapshots().await
    }

    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        info!("Deleting snapshot: {}", snapshot_id);

        // First check if snapshot exists
        let snapshots = self.db.list_snapshots().await?;
        let snapshot_exists = snapshots.iter().any(|s| s.id == snapshot_id);

        if !snapshot_exists {
            return Err(anyhow::anyhow!("Snapshot not found: {}", snapshot_id));
        }

        // Delete from database
        self.db.delete_snapshot(snapshot_id).await?;

        info!("✅ Successfully deleted snapshot: {}", snapshot_id);
        Ok(())
    }

    pub async fn revert_to_snapshot(&self, snapshot_id: &str) -> Result<()> {
        info!("Reverting to snapshot: {}", snapshot_id);

        let snapshots = self.db.list_snapshots().await?;
        let target_snapshot = snapshots
            .into_iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?;

        let current_packages = self.db.get_installed_packages().await?;
        let target_packages = &target_snapshot.packages;

        let (to_install, to_remove) = self.calculate_diff(&current_packages, target_packages);

        info!("Packages to remove: {}", to_remove.len());
        info!("Packages to install: {}", to_install.len());

        // Remove packages that shouldn't be in the target state
        for package in &to_remove {
            if let Err(e) = self.remove_package(package).await {
                warn!("Failed to remove package {}: {}", package.package_name, e);
            }
        }

        // Install packages that should be in the target state
        for package in &to_install {
            if let Err(e) = self.install_package(package).await {
                warn!("Failed to install package {}: {}", package.package_name, e);
            }
        }

        info!(
            "✅ Successfully reverted to snapshot '{}'",
            target_snapshot.name
        );
        Ok(())
    }

    fn calculate_diff(
        &self,
        current: &[InstallRecord],
        target: &[InstallRecord],
    ) -> (Vec<InstallRecord>, Vec<InstallRecord>) {
        let current_map: HashMap<String, &InstallRecord> = current
            .iter()
            .map(|p| (format!("{}:{}", p.package_name, p.box_type), p))
            .collect();

        let target_map: HashMap<String, &InstallRecord> = target
            .iter()
            .map(|p| (format!("{}:{}", p.package_name, p.box_type), p))
            .collect();

        let mut to_install = Vec::new();
        let mut to_remove = Vec::new();

        // Find packages to install (in target but not in current)
        for (key, package) in &target_map {
            if !current_map.contains_key(key) {
                to_install.push((*package).clone());
            }
        }

        // Find packages to remove (in current but not in target)
        for (key, package) in &current_map {
            if !target_map.contains_key(key) {
                to_remove.push((*package).clone());
            }
        }

        (to_install, to_remove)
    }

    async fn install_package(&self, package: &InstallRecord) -> Result<()> {
        info!(
            "Installing package {} via {}",
            package.package_name, package.box_type
        );

        match package.box_type.as_str() {
            "apt" if distro::command_exists("apt") => {
                if let Ok(apt_manager) = AptManager::new() {
                    apt_manager.install(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create apt manager"));
                }
            }
            "dnf" if distro::command_exists("dnf") => {
                if let Ok(dnf_manager) = DnfBox::new() {
                    dnf_manager.install(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create dnf manager"));
                }
            }
            "pacman" if distro::command_exists("pacman") => {
                if let Ok(pacman_manager) = PacmanBox::new() {
                    pacman_manager.install(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create pacman manager"));
                }
            }
            "flatpak" if distro::command_exists("flatpak") => {
                if let Ok(flatpak_manager) = FlatpakBox::new() {
                    let name = package
                        .source_url
                        .as_deref()
                        .unwrap_or(&package.package_name);
                    flatpak_manager.install(name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create flatpak manager"));
                }
            }
            "snap" if distro::command_exists("snap") => {
                if let Ok(snap_manager) = SnapBox::new() {
                    snap_manager.install(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create snap manager"));
                }
            }
            "appimage" => {
                if let Some(url) = &package.source_url {
                    appimage::install_appimage(url, &package.package_name).await?;
                } else {
                    return Err(anyhow::anyhow!("AppImage source URL not found"));
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported or unavailable box type: {}",
                    package.box_type
                ));
            }
        }

        // Record the installation
        let install_record = InstallRecord {
            id: Uuid::new_v4().to_string(),
            package_name: package.package_name.clone(),
            box_type: package.box_type.clone(),
            version: package.version.clone(),
            source_url: package.source_url.clone(),
            install_path: package.install_path.clone(),
            installed_at: Utc::now(),
            status: InstallStatus::Success,
            metadata: package.metadata.clone(),
        };

        self.db.record_install(&install_record).await?;

        Ok(())
    }

    async fn remove_package(&self, package: &InstallRecord) -> Result<()> {
        info!(
            "Removing package {} via {}",
            package.package_name, package.box_type
        );

        match package.box_type.as_str() {
            "apt" if distro::command_exists("apt") => {
                if let Ok(apt_manager) = AptManager::new() {
                    apt_manager.remove(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create apt manager"));
                }
            }
            "dnf" if distro::command_exists("dnf") => {
                if let Ok(dnf_manager) = DnfBox::new() {
                    dnf_manager.remove(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create dnf manager"));
                }
            }
            "pacman" if distro::command_exists("pacman") => {
                if let Ok(pacman_manager) = PacmanBox::new() {
                    pacman_manager.remove(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create pacman manager"));
                }
            }
            "flatpak" if distro::command_exists("flatpak") => {
                if let Ok(flatpak_manager) = FlatpakBox::new() {
                    flatpak_manager.remove(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create flatpak manager"));
                }
            }
            "snap" if distro::command_exists("snap") => {
                if let Ok(snap_manager) = SnapBox::new() {
                    snap_manager.remove(&package.package_name)?;
                } else {
                    return Err(anyhow::anyhow!("Failed to create snap manager"));
                }
            }
            "appimage" => {
                appimage::remove_appimage(&package.package_name)?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported or unavailable box type: {}",
                    package.box_type
                ));
            }
        }

        // Record the removal
        let removal_record = InstallRecord {
            id: Uuid::new_v4().to_string(),
            package_name: package.package_name.clone(),
            box_type: package.box_type.clone(),
            version: package.version.clone(),
            source_url: package.source_url.clone(),
            install_path: package.install_path.clone(),
            installed_at: Utc::now(),
            status: InstallStatus::Removed,
            metadata: package.metadata.clone(),
        };

        self.db.record_install(&removal_record).await?;

        Ok(())
    }

    pub async fn auto_snapshot(&self, operation: &str, package: &str) -> Result<Option<String>> {
        let snapshot_name = format!(
            "auto-{}-{}-{}",
            operation,
            package,
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        let description = Some(format!(
            "Automatic snapshot before {} {}",
            operation, package
        ));

        match self
            .create_snapshot(&snapshot_name, description.as_deref())
            .await
        {
            Ok(snapshot_id) => {
                info!("Created automatic snapshot: {}", snapshot_name);
                Ok(Some(snapshot_id))
            }
            Err(e) => {
                warn!("Failed to create automatic snapshot: {}", e);
                Ok(None)
            }
        }
    }
}
