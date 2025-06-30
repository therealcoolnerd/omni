use crate::boxes::{apt, dnf, flatpak, pacman, snap};
use crate::config::OmniConfig;
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::distro;
use anyhow::Result;
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct UpdateManager {
    db: Database,
    config: OmniConfig,
}

#[derive(Debug, Clone)]
pub struct UpdateCandidate {
    pub package_name: String,
    pub box_type: String,
    pub current_version: Option<String>,
    pub available_version: Option<String>,
    pub install_record: InstallRecord,
}

impl UpdateManager {
    pub async fn new(config: OmniConfig) -> Result<Self> {
        let db = Database::new().await?;
        Ok(Self { db, config })
    }

    pub async fn check_updates(&self) -> Result<Vec<UpdateCandidate>> {
        info!("Checking for available updates");

        let installed_packages = self.db.get_installed_packages().await?;
        let mut candidates = Vec::new();

        for package in installed_packages {
            if let Ok(candidate) = self.check_package_update(&package).await {
                if let Some(candidate) = candidate {
                    candidates.push(candidate);
                }
            }
        }

        info!("Found {} packages with available updates", candidates.len());
        Ok(candidates)
    }

    async fn check_package_update(
        &self,
        package: &InstallRecord,
    ) -> Result<Option<UpdateCandidate>> {
        match package.box_type.as_str() {
            "apt" if distro::command_exists("apt") => self.check_apt_update(package).await,
            "dnf" if distro::command_exists("dnf") => self.check_dnf_update(package).await,
            "pacman" if distro::command_exists("pacman") => self.check_pacman_update(package).await,
            "snap" if distro::command_exists("snap") => self.check_snap_update(package).await,
            "flatpak" if distro::command_exists("flatpak") => {
                self.check_flatpak_update(package).await
            }
            _ => Ok(None),
        }
    }

    async fn check_apt_update(&self, package: &InstallRecord) -> Result<Option<UpdateCandidate>> {
        let output = Command::new("apt")
            .arg("list")
            .arg("--upgradable")
            .arg(&package.package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                // Skip header
                if line.contains(&package.package_name) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return Ok(Some(UpdateCandidate {
                            package_name: package.package_name.clone(),
                            box_type: package.box_type.clone(),
                            current_version: package.version.clone(),
                            available_version: Some(parts[1].to_string()),
                            install_record: package.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn check_dnf_update(&self, package: &InstallRecord) -> Result<Option<UpdateCandidate>> {
        let output = Command::new("dnf")
            .arg("check-update")
            .arg(&package.package_name)
            .output()?;

        // dnf check-update returns 100 if updates are available
        if output.status.code() == Some(100) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with(&package.package_name) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return Ok(Some(UpdateCandidate {
                            package_name: package.package_name.clone(),
                            box_type: package.box_type.clone(),
                            current_version: package.version.clone(),
                            available_version: Some(parts[1].to_string()),
                            install_record: package.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn check_pacman_update(
        &self,
        package: &InstallRecord,
    ) -> Result<Option<UpdateCandidate>> {
        let output = Command::new("pacman")
            .arg("-Qu")
            .arg(&package.package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(&package.package_name) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        return Ok(Some(UpdateCandidate {
                            package_name: package.package_name.clone(),
                            box_type: package.box_type.clone(),
                            current_version: Some(parts[1].to_string()),
                            available_version: Some(parts[3].to_string()),
                            install_record: package.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn check_snap_update(&self, package: &InstallRecord) -> Result<Option<UpdateCandidate>> {
        let output = Command::new("snap").arg("refresh").arg("--list").output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 && parts[0] == package.package_name {
                    return Ok(Some(UpdateCandidate {
                        package_name: package.package_name.clone(),
                        box_type: package.box_type.clone(),
                        current_version: Some(parts[2].to_string()),
                        available_version: Some(parts[4].to_string()),
                        install_record: package.clone(),
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn check_flatpak_update(
        &self,
        package: &InstallRecord,
    ) -> Result<Option<UpdateCandidate>> {
        let output = Command::new("flatpak")
            .arg("remote-info")
            .arg("--show-commit")
            .arg(
                package
                    .source_url
                    .as_deref()
                    .unwrap_or(&package.package_name),
            )
            .output()?;

        if output.status.success() {
            // For Flatpak, we just indicate an update is available
            // The actual version comparison is complex with Flatpak
            return Ok(Some(UpdateCandidate {
                package_name: package.package_name.clone(),
                box_type: package.box_type.clone(),
                current_version: package.version.clone(),
                available_version: Some("latest".to_string()),
                install_record: package.clone(),
            }));
        }

        Ok(None)
    }

    pub async fn update_package(&self, candidate: &UpdateCandidate) -> Result<()> {
        info!(
            "Updating package: {} via {}",
            candidate.package_name, candidate.box_type
        );

        let pb = ProgressBar::new_spinner();
        if let Ok(style) = ProgressStyle::default_spinner().template("{spinner:.green} {msg}") {
            pb.set_style(style);
        }
        pb.set_message(format!("Updating {}...", candidate.package_name));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let result = match candidate.box_type.as_str() {
            "apt" => self.update_apt_package(&candidate.package_name).await,
            "dnf" => self.update_dnf_package(&candidate.package_name).await,
            "pacman" => self.update_pacman_package(&candidate.package_name).await,
            "snap" => self.update_snap_package(&candidate.package_name).await,
            "flatpak" => self.update_flatpak_package(&candidate).await,
            _ => {
                error!("Unsupported box type for update: {}", candidate.box_type);
                Err(anyhow::anyhow!("Unsupported box type"))
            }
        };

        pb.finish_and_clear();

        match result {
            Ok(_) => {
                info!("✅ Successfully updated {}", candidate.package_name);

                // Record the update
                let update_record = InstallRecord {
                    id: Uuid::new_v4().to_string(),
                    package_name: candidate.package_name.clone(),
                    box_type: candidate.box_type.clone(),
                    version: candidate.available_version.clone(),
                    source_url: candidate.install_record.source_url.clone(),
                    install_path: candidate.install_record.install_path.clone(),
                    installed_at: Utc::now(),
                    status: InstallStatus::Updated,
                    metadata: Some(format!(
                        "Updated from version {:?}",
                        candidate.current_version
                    )),
                };

                self.db.record_install(&update_record).await?;
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update {}: {}", candidate.package_name, e);
                Err(e)
            }
        }
    }

    async fn update_apt_package(&self, package_name: &str) -> Result<()> {
        let output = Command::new("apt")
            .arg("install")
            .arg("--only-upgrade")
            .arg("-y")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("APT update failed: {}", error_msg))
        }
    }

    async fn update_dnf_package(&self, package_name: &str) -> Result<()> {
        let output = Command::new("dnf")
            .arg("update")
            .arg("-y")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("DNF update failed: {}", error_msg))
        }
    }

    async fn update_pacman_package(&self, package_name: &str) -> Result<()> {
        let output = Command::new("pacman")
            .arg("-S")
            .arg("--noconfirm")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Pacman update failed: {}", error_msg))
        }
    }

    async fn update_snap_package(&self, package_name: &str) -> Result<()> {
        snap::update_snap(package_name)
    }

    async fn update_flatpak_package(&self, candidate: &UpdateCandidate) -> Result<()> {
        let package_ref = candidate
            .install_record
            .source_url
            .as_deref()
            .unwrap_or(&candidate.package_name);

        let output = Command::new("flatpak")
            .arg("update")
            .arg("-y")
            .arg(package_ref)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Flatpak update failed: {}", error_msg))
        }
    }

    pub async fn update_all(&self) -> Result<()> {
        info!("Starting system-wide update");

        let candidates = self.check_updates().await?;

        if candidates.is_empty() {
            info!("✅ All packages are up to date");
            return Ok(());
        }

        info!("Updating {} packages", candidates.len());

        let pb = ProgressBar::new(candidates.len() as u64);
        if let Ok(style) = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        {
            pb.set_style(style.progress_chars("#>-"));
        }

        for (i, candidate) in candidates.iter().enumerate() {
            pb.set_message(format!("Updating {}", candidate.package_name));

            if let Err(e) = self.update_package(candidate).await {
                warn!("Failed to update {}: {}", candidate.package_name, e);
            }

            pb.set_position(i as u64 + 1);
        }

        pb.finish_with_message("✅ Update complete");
        info!("✅ System update completed");

        Ok(())
    }

    pub async fn refresh_repositories(&self) -> Result<()> {
        info!("Refreshing package repositories");

        // Update apt repositories
        if distro::command_exists("apt") {
            info!("Updating apt repositories");
            let _ = Command::new("apt").arg("update").output();
        }

        // Update dnf cache
        if distro::command_exists("dnf") {
            info!("Updating dnf cache");
            let _ = Command::new("dnf").arg("makecache").output();
        }

        // Update pacman databases
        if distro::command_exists("pacman") {
            info!("Updating pacman databases");
            let _ = Command::new("pacman").arg("-Sy").output();
        }

        // Refresh flatpak repositories
        if distro::command_exists("flatpak") {
            info!("Refreshing flatpak repositories");
            let _ = Command::new("flatpak")
                .arg("update")
                .arg("--appstream")
                .output();
        }

        info!("✅ Repository refresh completed");
        Ok(())
    }

    pub async fn list_installed(&self) -> Result<Vec<InstallRecord>> {
        self.db.get_installed_packages().await
    }
}
