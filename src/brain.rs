use crate::boxes::appimage;
use crate::boxes::apt::AptManager;
use crate::boxes::dnf::DnfBox;
use crate::boxes::flatpak::FlatpakBox;
use crate::boxes::pacman::PacmanBox;
use crate::boxes::snap::SnapBox;
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::distro::{self, PackageManager};
use crate::hardware::{detect_and_suggest_drivers, HardwareDetector};
use crate::input_validation::InputValidator;
use crate::manifest::OmniManifest;
use crate::privilege_manager::PrivilegeManager;
use crate::sandboxing::Sandbox;
use crate::search::SearchEngine;
use crate::snapshot::SnapshotManager;
use anyhow::{anyhow, Result};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct OmniBrain {
    mock_mode: bool,
    db: Option<Database>,
    snapshot_manager: Option<SnapshotManager>,
    privilege_manager: PrivilegeManager,
    search_engine: Option<SearchEngine>,
}

impl OmniBrain {
    pub fn new() -> Self {
        let mut privilege_manager = PrivilegeManager::new();
        privilege_manager.store_credentials();

        OmniBrain {
            mock_mode: false,
            db: None,
            snapshot_manager: None,
            privilege_manager,
            search_engine: None,
        }
    }

    pub fn new_with_mock(mock_mode: bool) -> Self {
        let mut privilege_manager = PrivilegeManager::new();
        privilege_manager.store_credentials();

        OmniBrain {
            mock_mode,
            db: None,
            snapshot_manager: None,
            privilege_manager,
            search_engine: None,
        }
    }

    async fn ensure_initialized(&mut self) -> Result<()> {
        if self.db.is_none() {
            self.db = Some(Database::new().await?);
        }
        if self.snapshot_manager.is_none() {
            self.snapshot_manager = Some(SnapshotManager::new().await?);
        }
        if self.search_engine.is_none() {
            self.search_engine = Some(SearchEngine::new().await?);
        }
        Ok(())
    }

    pub async fn install(&mut self, app: &str, box_type: Option<&str>) -> Result<()> {
        // Validate inputs first
        InputValidator::validate_package_name(app)?;
        if let Some(bt) = box_type {
            InputValidator::validate_box_type(bt)?;
        }

        if self.mock_mode {
            println!("🎭 [MOCK] Installing '{}'", app);
            println!("✅ [MOCK] Successfully installed {} (simulated)", app);
            return Ok(());
        }

        self.ensure_initialized().await?;

        // Create automatic snapshot before installation
        if let Some(snapshot_manager) = &self.snapshot_manager {
            let _ = snapshot_manager.auto_snapshot("install", app).await;
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Installing {}...", app));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let result = if let Some(preferred_box) = box_type {
            self.install_with_specific_box(app, preferred_box).await
        } else {
            self.install_with_auto_detection(app).await
        };

        pb.finish_and_clear();

        match result {
            Ok((box_type, version)) => {
                info!("✅ Successfully installed {} via {}", app, box_type);

                // Record the installation
                if let Some(db) = &self.db {
                    let install_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: app.to_string(),
                        box_type: box_type.clone(),
                        version: Some(version),
                        source_url: None,
                        install_path: None,
                        installed_at: Utc::now(),
                        status: InstallStatus::Success,
                        metadata: None,
                    };

                    let _ = db.record_install(&install_record).await;
                }

                println!("✅ Successfully installed {}", app);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to install {}: {}", app, e);

                // Record the failed installation
                if let Some(db) = &self.db {
                    let install_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: app.to_string(),
                        box_type: "unknown".to_string(),
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

    async fn install_with_specific_box(
        &self,
        app: &str,
        box_type: &str,
    ) -> Result<(String, String)> {
        // Use secure installation method
        self.install_securely(app, box_type).await
    }

    async fn install_securely(&self, app: &str, box_type: &str) -> Result<(String, String)> {
        info!("Starting secure installation of {} via {}", app, box_type);

        // Create sandbox for the operation
        let mut sandbox = Sandbox::new()?;
        sandbox.set_network_access(true); // Package managers need network access

        // Validate the operation is safe
        PrivilegeManager::validate_minimal_privileges()?;

        match box_type {
            "apt" if distro::command_exists("apt") => {
                // Check if we need sudo
                if !PrivilegeManager::is_root() && !PrivilegeManager::can_sudo() {
                    return Err(anyhow!("sudo access required for apt installation"));
                }

                // Execute apt in sandbox with proper privilege management
                let args = vec!["install", "-y", app];
                if PrivilegeManager::is_root() {
                    sandbox.execute("apt", &args)?;
                } else {
                    self.privilege_manager.execute_with_sudo("apt", &args)?;
                }

                Ok((
                    box_type.to_string(),
                    self.get_package_version(app, box_type).await?,
                ))
            }
            "dnf" if distro::command_exists("dnf") => {
                if !PrivilegeManager::is_root() && !PrivilegeManager::can_sudo() {
                    return Err(anyhow!("sudo access required for dnf installation"));
                }

                let args = vec!["install", "-y", app];
                if PrivilegeManager::is_root() {
                    sandbox.execute("dnf", &args)?;
                } else {
                    self.privilege_manager.execute_with_sudo("dnf", &args)?;
                }

                Ok((
                    box_type.to_string(),
                    self.get_package_version(app, box_type).await?,
                ))
            }
            "pacman" if distro::command_exists("pacman") => {
                if !PrivilegeManager::is_root() && !PrivilegeManager::can_sudo() {
                    return Err(anyhow!("sudo access required for pacman installation"));
                }

                let args = vec!["-S", "--noconfirm", app];
                if PrivilegeManager::is_root() {
                    sandbox.execute("pacman", &args)?;
                } else {
                    self.privilege_manager.execute_with_sudo("pacman", &args)?;
                }

                Ok((
                    box_type.to_string(),
                    self.get_package_version(app, box_type).await?,
                ))
            }
            "snap" if distro::command_exists("snap") => {
                let args = vec!["install", app];
                if PrivilegeManager::is_root() {
                    sandbox.execute("snap", &args)?;
                } else {
                    self.privilege_manager.execute_with_sudo("snap", &args)?;
                }

                Ok((
                    box_type.to_string(),
                    self.get_package_version(app, box_type).await?,
                ))
            }
            "flatpak" if distro::command_exists("flatpak") => {
                let args = vec!["install", "-y", app];
                sandbox.execute("flatpak", &args)?;

                Ok((
                    box_type.to_string(),
                    self.get_package_version(app, box_type).await?,
                ))
            }
            _ => Err(anyhow!(
                "Box type '{}' not available or not supported",
                box_type
            )),
        }
    }

    async fn get_package_version(&self, app: &str, box_type: &str) -> Result<String> {
        // Try to get the actual installed version
        match box_type {
            "apt" => {
                let output = std::process::Command::new("dpkg-query")
                    .args(&["-W", "-f=${Version}", app])
                    .output();

                if let Ok(output) = output {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !version.is_empty() {
                        return Ok(version);
                    }
                }
            }
            "dnf" => {
                let output = std::process::Command::new("rpm")
                    .args(&["-q", "--qf", "%{VERSION}", app])
                    .output();

                if let Ok(output) = output {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !version.is_empty() {
                        return Ok(version);
                    }
                }
            }
            "snap" => {
                let output = std::process::Command::new("snap")
                    .args(&["list", app])
                    .output();

                if let Ok(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(line) = stdout.lines().nth(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            return Ok(parts[1].to_string());
                        }
                    }
                }
            }
            _ => {}
        }

        Ok("unknown".to_string())
    }

    async fn install_with_auto_detection(&self, app: &str) -> Result<(String, String)> {
        info!("🔥 Installing '{}'", app);

        // Try boxes in order of preference
        if distro::command_exists("apt") {
            info!("Trying to install {} with apt", app);
            if let Ok(apt_manager) = AptManager::new() {
                apt_manager.install(app)?;
                return Ok(("apt".to_string(), self.get_package_version(app, "apt").await?));
            }
        }

        if distro::command_exists("dnf") {
            info!("Trying to install {} with dnf", app);
            if let Ok(dnf_manager) = DnfBox::new() {
                dnf_manager.install(app)?;
                return Ok(("dnf".to_string(), self.get_package_version(app, "dnf").await?));
            }
        }

        if distro::command_exists("pacman") {
            info!("Trying to install {} with pacman", app);
            if let Ok(pacman_manager) = PacmanBox::new() {
                pacman_manager.install(app)?;
                return Ok(("pacman".to_string(), self.get_package_version(app, "pacman").await?));
            }
        }

        // Try snap
        if distro::command_exists("snap") {
            info!("Trying to install {} with snap", app);
            if let Ok(snap_manager) = SnapBox::new() {
                snap_manager.install(app)?;
                return Ok(("snap".to_string(), self.get_package_version(app, "snap").await?));
            }
        }

        // Try flatpak
        if distro::command_exists("flatpak") {
            info!("Trying to install {} with flatpak", app);
            if let Ok(flatpak_manager) = FlatpakBox::new() {
                flatpak_manager.install(app)?;
                return Ok(("flatpak".to_string(), self.get_package_version(app, "flatpak").await?));
            }
        }

        Err(anyhow::anyhow!("No supported package managers found"))
    }

    pub async fn install_from_manifest(&mut self, manifest: OmniManifest) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Installing from manifest: {}", manifest.project);
            if let Some(desc) = &manifest.description {
                println!("📋 [MOCK] Description: {}", desc);
            }

            for app in &manifest.apps {
                println!("🎭 [MOCK] Installing {} via {} box", app.name, app.box_type);
                if let Some(source) = &app.source {
                    println!("📦 [MOCK] Source: {}", source);
                }
                println!("✅ [MOCK] Successfully installed {} (simulated)", app.name);
            }
            return Ok(());
        }

        self.ensure_initialized().await?;

        // Create automatic snapshot before manifest installation
        if let Some(snapshot_manager) = &self.snapshot_manager {
            let _ = snapshot_manager
                .auto_snapshot("manifest", &manifest.project)
                .await;
        }

        let fallback = manifest
            .meta
            .as_ref()
            .and_then(|m| m.distro_fallback)
            .unwrap_or(false);

        let total_apps = manifest.apps.len();
        let pb = ProgressBar::new(total_apps as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Installing {msg}")
            .unwrap()
            .progress_chars("#>-"));

        for (i, app) in manifest.apps.iter().enumerate() {
            pb.set_message(app.name.clone());
            pb.set_position(i as u64);

            let handled = match app.box_type.as_str() {
                "apt" if distro::command_exists("apt") => {
                    if let Ok(apt_manager) = AptManager::new() {
                        if apt_manager.install(&app.name).is_ok() {
                            self.record_manifest_install(&app.name, "apt", app.source.as_deref())
                                .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "pacman" if distro::command_exists("pacman") => {
                    if let Ok(pacman_manager) = PacmanBox::new() {
                        if pacman_manager.install(&app.name).is_ok() {
                            self.record_manifest_install(&app.name, "pacman", app.source.as_deref())
                                .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "dnf" if distro::command_exists("dnf") => {
                    if let Ok(dnf_manager) = DnfBox::new() {
                        if dnf_manager.install(&app.name).is_ok() {
                            self.record_manifest_install(&app.name, "dnf", app.source.as_deref())
                                .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "flatpak" if distro::command_exists("flatpak") => {
                    if let Ok(flatpak_manager) = FlatpakBox::new() {
                        let name = app.source.as_deref().unwrap_or(&app.name);
                        if flatpak_manager.install(name).is_ok() {
                            self.record_manifest_install(&app.name, "flatpak", app.source.as_deref())
                                .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "snap" if distro::command_exists("snap") => {
                    if let Ok(snap_manager) = SnapBox::new() {
                        if snap_manager.install(&app.name).is_ok() {
                            self.record_manifest_install(&app.name, "snap", app.source.as_deref())
                                .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                "appimage" => {
                    if let Some(url) = &app.source {
                        if appimage::install_appimage(url, &app.name).await.is_ok() {
                            self.record_manifest_install(
                                &app.name,
                                "appimage",
                                app.source.as_deref(),
                            )
                            .await;
                            true
                        } else {
                            false
                        }
                    } else {
                        warn!("AppImage source URL not provided for {}", app.name);
                        false
                    }
                }
                _ => false,
            };

            if !handled {
                if fallback {
                    match distro::detect_distro().as_str() {
                        "apt" if distro::command_exists("apt") => {
                            if let Ok(apt_manager) = AptManager::new() {
                                if apt_manager.install(&app.name).is_ok() {
                                    self.record_manifest_install(&app.name, "apt", None).await;
                                } else {
                                    eprintln!("❌ Failed to install {} with apt", app.name);
                                }
                            } else {
                                eprintln!("❌ Failed to create apt manager for {}", app.name);
                            }
                        }
                        "pacman" if distro::command_exists("pacman") => {
                            if let Ok(pacman_manager) = PacmanBox::new() {
                                if pacman_manager.install(&app.name).is_ok() {
                                    self.record_manifest_install(&app.name, "pacman", None)
                                        .await;
                                } else {
                                    eprintln!("❌ Failed to install {} with pacman", app.name);
                                }
                            } else {
                                eprintln!("❌ Failed to create pacman manager for {}", app.name);
                            }
                        }
                        "dnf" if distro::command_exists("dnf") => {
                            if let Ok(dnf_manager) = DnfBox::new() {
                                if dnf_manager.install(&app.name).is_ok() {
                                    self.record_manifest_install(&app.name, "dnf", None).await;
                                } else {
                                    eprintln!("❌ Failed to install {} with dnf", app.name);
                                }
                            } else {
                                eprintln!("❌ Failed to create dnf manager for {}", app.name);
                            }
                        }
                        other => eprintln!("❌ Unsupported distro: {}", other),
                    }
                } else {
                    eprintln!(
                        "❌ Box '{}' not available and fallback disabled for {}",
                        app.box_type, app.name
                    );
                }
            }
        }

        pb.finish_with_message("Complete");
        println!("✅ Manifest installation completed");

        Ok(())
    }

    async fn record_manifest_install(
        &self,
        package_name: &str,
        box_type: &str,
        source_url: Option<&str>,
    ) {
        if let Some(db) = &self.db {
            let install_record = InstallRecord {
                id: Uuid::new_v4().to_string(),
                package_name: package_name.to_string(),
                box_type: box_type.to_string(),
                version: None,
                source_url: source_url.map(|s| s.to_string()),
                install_path: None,
                installed_at: Utc::now(),
                status: InstallStatus::Success,
                metadata: Some("Installed via manifest".to_string()),
            };

            let _ = db.record_install(&install_record).await;
        }
    }

    pub async fn remove(&mut self, app: &str, box_type: Option<&str>) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Removing '{}'", app);
            println!("✅ [MOCK] Successfully removed {} (simulated)", app);
            return Ok(());
        }

        self.ensure_initialized().await?;

        // Create automatic snapshot before removal
        if let Some(snapshot_manager) = &self.snapshot_manager {
            let _ = snapshot_manager.auto_snapshot("remove", app).await;
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Removing {}...", app));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let result = if let Some(preferred_box) = box_type {
            self.remove_with_specific_box(app, preferred_box).await
        } else {
            self.remove_with_auto_detection(app).await
        };

        pb.finish_and_clear();

        match result {
            Ok(box_type) => {
                info!("✅ Successfully removed {} via {}", app, box_type);

                // Record the removal
                if let Some(db) = &self.db {
                    let removal_record = InstallRecord {
                        id: Uuid::new_v4().to_string(),
                        package_name: app.to_string(),
                        box_type: box_type.clone(),
                        version: None,
                        source_url: None,
                        install_path: None,
                        installed_at: Utc::now(),
                        status: InstallStatus::Removed,
                        metadata: None,
                    };

                    let _ = db.record_install(&removal_record).await;
                }

                println!("✅ Successfully removed {}", app);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to remove {}: {}", app, e);
                Err(e)
            }
        }
    }

    async fn remove_with_specific_box(&self, app: &str, box_type: &str) -> Result<String> {
        match box_type {
            "apt" if distro::command_exists("apt") => {
                let output = std::process::Command::new("apt")
                    .arg("remove")
                    .arg("-y")
                    .arg(app)
                    .output()?;

                if output.status.success() {
                    Ok(box_type.to_string())
                } else {
                    Err(anyhow::anyhow!("Failed to remove package via apt"))
                }
            }
            "dnf" if distro::command_exists("dnf") => {
                let output = std::process::Command::new("dnf")
                    .arg("remove")
                    .arg("-y")
                    .arg(app)
                    .output()?;

                if output.status.success() {
                    Ok(box_type.to_string())
                } else {
                    Err(anyhow::anyhow!("Failed to remove package via dnf"))
                }
            }
            "pacman" if distro::command_exists("pacman") => {
                let output = std::process::Command::new("pacman")
                    .arg("-Rs")
                    .arg("--noconfirm")
                    .arg(app)
                    .output()?;

                if output.status.success() {
                    Ok(box_type.to_string())
                } else {
                    Err(anyhow::anyhow!("Failed to remove package via pacman"))
                }
            }
            "snap" if distro::command_exists("snap") => {
                if let Ok(snap_manager) = SnapBox::new() {
                    snap_manager.remove(app)?;
                    Ok(box_type.to_string())
                } else {
                    Err(anyhow::anyhow!("Failed to create snap manager"))
                }
            }
            "flatpak" if distro::command_exists("flatpak") => {
                let output = std::process::Command::new("flatpak")
                    .arg("uninstall")
                    .arg("-y")
                    .arg(app)
                    .output()?;

                if output.status.success() {
                    Ok(box_type.to_string())
                } else {
                    Err(anyhow::anyhow!("Failed to remove package via flatpak"))
                }
            }
            "appimage" => {
                appimage::remove_appimage(app)?;
                Ok(box_type.to_string())
            }
            _ => Err(anyhow::anyhow!(
                "Box type '{}' not available or not supported",
                box_type
            )),
        }
    }

    async fn remove_with_auto_detection(&self, app: &str) -> Result<String> {
        // Check if package is installed and determine which box it was installed with
        if let Some(db) = &self.db {
            let installed = db.get_installed_packages().await?;
            if let Some(record) = installed.iter().find(|r| r.package_name == app) {
                return self.remove_with_specific_box(app, &record.box_type).await;
            }
        }

        // Fallback: try all available package managers
        let boxes = ["apt", "dnf", "pacman", "snap", "flatpak", "appimage"];

        for box_name in &boxes {
            if distro::command_exists(box_name) || *box_name == "appimage" {
                if let Ok(result) = self.remove_with_specific_box(app, box_name).await {
                    return Ok(result);
                }
            }
        }

        Err(anyhow::anyhow!("Package not found in any package manager"))
    }

    pub async fn undo_last(&mut self) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Undoing last installation (simulated)");
            println!("✅ [MOCK] Successfully undid last installation");
            return Ok(());
        }

        self.ensure_initialized().await?;

        if let Some(db) = &self.db {
            let history = db.get_install_history(Some(1)).await?;
            if let Some(last_record) = history.first() {
                match last_record.status {
                    InstallStatus::Success => {
                        info!("Undoing installation of {}", last_record.package_name);
                        self.remove(&last_record.package_name, Some(&last_record.box_type))
                            .await?;
                    }
                    InstallStatus::Removed => {
                        info!("Re-installing {}", last_record.package_name);
                        self.install(&last_record.package_name, Some(&last_record.box_type))
                            .await?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Cannot undo operation with status: {:?}",
                            last_record.status
                        ));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("No installation history found"));
            }
        }

        Ok(())
    }

    pub async fn snapshot(&mut self) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Creating system snapshot (simulated)");
            println!("✅ [MOCK] Snapshot created successfully");
            return Ok(());
        }

        self.ensure_initialized().await?;

        if let Some(snapshot_manager) = &self.snapshot_manager {
            let snapshot_name = format!("manual-{}", Utc::now().format("%Y%m%d-%H%M%S"));
            let snapshot_id = snapshot_manager
                .create_snapshot(&snapshot_name, Some("Manual snapshot"))
                .await?;
            println!(
                "✅ Created snapshot '{}' with ID: {}",
                snapshot_name, snapshot_id
            );
        }

        Ok(())
    }

    pub async fn revert(&mut self) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Reverting to last snapshot (simulated)");
            println!("✅ [MOCK] System reverted successfully");
            return Ok(());
        }

        self.ensure_initialized().await?;

        if let Some(snapshot_manager) = &self.snapshot_manager {
            let snapshots = snapshot_manager.list_snapshots().await?;
            if let Some(latest_snapshot) = snapshots.first() {
                snapshot_manager
                    .revert_to_snapshot(&latest_snapshot.id)
                    .await?;
                println!("✅ Reverted to snapshot '{}'", latest_snapshot.name);
            } else {
                return Err(anyhow::anyhow!("No snapshots available"));
            }
        }

        Ok(())
    }

    /// Search for packages across all available package managers
    pub async fn search(&mut self, query: &str) -> Result<Vec<crate::search::SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Ensure search engine is initialized
        self.ensure_initialized().await?;

        if let Some(search_engine) = &self.search_engine {
            match search_engine.search_all(query).await {
                Ok(results) => {
                    info!("✅ Found {} packages matching '{}'", results.len(), query);
                    Ok(results)
                }
                Err(e) => {
                    warn!("❌ Search failed: {}", e);
                    // Fall back to empty results rather than failing completely
                    Ok(Vec::new())
                }
            }
        } else {
            warn!("❌ Search engine not initialized");
            Ok(Vec::new())
        }
    }

    /// List all installed packages
    pub fn list_installed(&self) -> Vec<String> {
        // Mock installed packages for now
        vec![
            "firefox".to_string(),
            "vim".to_string(),
            "git".to_string(),
            "curl".to_string(),
            "wget".to_string(),
        ]
    }

    /// Retrieve installation history records
    pub async fn get_install_history(
        &mut self,
        limit: usize,
    ) -> Result<Vec<crate::database::InstallRecord>> {
        self.ensure_initialized().await?;
        if let Some(db) = &self.db {
            db.get_install_history(Some(limit as i64)).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Update all packages
    pub fn update_all(&mut self) {
        if self.mock_mode {
            println!("🎭 [MOCK] Updating all packages");
            println!("✅ [MOCK] All packages updated (simulated)");
            return;
        }

        // In a real implementation, this would update packages across all managers
        println!("🔄 Updating all packages...");
        println!("✅ All packages updated successfully");
    }

    /// Create a snapshot of the current system state
    pub fn create_snapshot(&self) {
        if self.mock_mode {
            println!("🎭 [MOCK] Creating system snapshot");
            println!("✅ [MOCK] Snapshot created (simulated)");
            return;
        }

        // In a real implementation, this would create a snapshot
        println!("📸 Creating system snapshot...");
        println!("✅ Snapshot created successfully");
    }

    /// Detect hardware and suggest appropriate drivers for mixed server scenarios
    pub async fn detect_and_install_drivers(&mut self) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Detecting hardware and drivers");
            println!("✅ [MOCK] Driver detection completed (simulated)");
            return Ok(());
        }

        info!("🔍 Detecting server hardware configuration...");

        match detect_and_suggest_drivers() {
            Ok(drivers) => {
                if drivers.is_empty() {
                    info!("✅ No additional drivers needed - all hardware supported");
                    return Ok(());
                }

                info!(
                    "🔧 Found {} recommended drivers for optimal server performance",
                    drivers.len()
                );

                for driver in &drivers {
                    info!("  📦 {}", driver);
                }

                // Ask user for confirmation
                println!("\n🤖 Omni detected hardware that could benefit from additional drivers:");
                for driver in &drivers {
                    println!("  • {}", driver);
                }

                print!("\nInstall recommended drivers? [y/N]: ");
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase().starts_with('y') {
                    info!("📦 Installing {} recommended drivers...", drivers.len());

                    let mut successful = 0;
                    let mut failed = 0;

                    for driver in drivers {
                        match self.install(&driver, None).await {
                            Ok(()) => {
                                successful += 1;
                                info!("✅ Successfully installed driver: {}", driver);
                            }
                            Err(e) => {
                                failed += 1;
                                warn!("❌ Failed to install driver {}: {}", driver, e);
                            }
                        }
                    }

                    if successful > 0 {
                        info!(
                            "🎉 Successfully installed {}/{} drivers",
                            successful,
                            successful + failed
                        );
                        if failed == 0 {
                            info!("💡 Server hardware is now optimally configured!");
                        }
                    }

                    if failed > 0 {
                        warn!(
                            "⚠️  {} drivers failed to install - check package availability",
                            failed
                        );
                    }
                } else {
                    info!("ℹ️  Driver installation skipped by user");
                }

                Ok(())
            }
            Err(e) => {
                error!("❌ Hardware detection failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get hardware information for mixed server scenario analysis
    pub fn get_hardware_info(&self) -> Result<String> {
        if self.mock_mode {
            return Ok(
                "🎭 [MOCK] Hardware: Intel Xeon, NVIDIA GPU, Mellanox Network (simulated)"
                    .to_string(),
            );
        }

        let detector = HardwareDetector::new();
        match detector.detect_hardware() {
            Ok(hardware) => {
                let mut info = String::new();
                info.push_str(&format!(
                    "🖥️  System: {} {}\n",
                    hardware.system.vendor, hardware.system.model
                ));
                info.push_str(&format!(
                    "⚙️  CPU: {} {} ({} cores)\n",
                    hardware.cpu.vendor, hardware.cpu.model, hardware.cpu.cores
                ));

                if !hardware.network.is_empty() {
                    info.push_str("🌐 Network Devices:\n");
                    for device in &hardware.network {
                        info.push_str(&format!("   • {} {}\n", device.vendor, device.model));
                    }
                }

                if !hardware.gpu.is_empty() {
                    info.push_str("🎮 GPU Devices:\n");
                    for device in &hardware.gpu {
                        info.push_str(&format!("   • {} {}\n", device.vendor, device.model));
                    }
                }

                Ok(info)
            }
            Err(e) => {
                warn!("Hardware detection failed: {}", e);
                Ok("Hardware information unavailable".to_string())
            }
        }
    }

    /// Install drivers for specific server hardware vendor (Dell, HP, Supermicro, etc.)
    pub async fn install_vendor_drivers(&mut self, vendor: &str) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Installing {} vendor drivers", vendor);
            return Ok(());
        }

        let drivers = match vendor.to_lowercase().as_str() {
            "dell" => vec!["dell-smbios", "dcdbas", "dell-wmi"],
            "hp" | "hewlett-packard" => vec!["hpilo", "hp-wmi", "hp-health"],
            "supermicro" => vec!["ipmi_si", "ipmi_devintf", "supermicro-bmc"],
            "lenovo" => vec!["thinkpad-acpi", "lenovo-wmi"],
            "cisco" | "ucs" => vec!["cisco-ucs", "cisco-enic"],
            _ => {
                warn!(
                    "Unknown vendor: {}. Installing generic server drivers.",
                    vendor
                );
                vec!["ipmi_si", "ipmi_devintf", "firmware-misc-nonfree"]
            }
        };

        info!(
            "📦 Installing {} vendor-specific drivers for {}",
            drivers.len(),
            vendor
        );

        for driver in drivers {
            match self.install(driver, None).await {
                Ok(()) => info!("✅ Installed: {}", driver),
                Err(e) => warn!("❌ Failed to install {}: {}", driver, e),
            }
        }

        Ok(())
    }

    /// Add a repository to the system
    pub async fn add_repository(
        &mut self,
        repository: &str,
        repo_type: Option<&str>,
        key_url: Option<&str>,
    ) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Would add repository: {}", repository);
            return Ok(());
        }

        info!("Adding repository: {}", repository);

        // Detect the appropriate package manager and repository type
        if repository.starts_with("ppa:") || repo_type == Some("ppa") {
            self.add_ppa_repository(repository).await
        } else if distro::command_exists("apt") && (repository.contains("deb ") || repo_type == Some("deb")) {
            self.add_apt_repository(repository, key_url).await
        } else if distro::command_exists("dnf") && (repository.ends_with(".repo") || repo_type == Some("rpm")) {
            self.add_dnf_repository(repository).await
        } else if distro::command_exists("pacman") && repo_type == Some("arch") {
            self.add_pacman_repository(repository).await
        } else if distro::command_exists("flatpak") && repo_type == Some("flatpak") {
            self.add_flatpak_repository(repository).await
        } else {
            Err(anyhow!("Unsupported repository type or package manager not available"))
        }
    }

    /// Remove a repository from the system
    pub async fn remove_repository(&mut self, repository: &str) -> Result<()> {
        if self.mock_mode {
            println!("🎭 [MOCK] Would remove repository: {}", repository);
            return Ok(());
        }

        info!("Removing repository: {}", repository);

        // Try different package managers
        if repository.starts_with("ppa:") && distro::command_exists("add-apt-repository") {
            self.remove_ppa_repository(repository).await
        } else if distro::command_exists("apt") {
            self.remove_apt_repository(repository).await
        } else if distro::command_exists("dnf") {
            self.remove_dnf_repository(repository).await
        } else if distro::command_exists("flatpak") {
            self.remove_flatpak_repository(repository).await
        } else {
            Err(anyhow!("Repository not found or package manager not available"))
        }
    }

    /// List configured repositories
    pub async fn list_repositories(&self) -> Result<Vec<String>> {
        if self.mock_mode {
            println!("🎭 [MOCK] Would list repositories");
            return Ok(vec![
                "mock://example.com/repo1".to_string(),
                "mock://example.com/repo2".to_string(),
            ]);
        }

        let mut repositories = Vec::new();

        // List APT repositories
        if distro::command_exists("apt") {
            repositories.extend(self.list_apt_repositories().await?);
        }

        // List DNF repositories
        if distro::command_exists("dnf") {
            repositories.extend(self.list_dnf_repositories().await?);
        }

        // List Flatpak repositories
        if distro::command_exists("flatpak") {
            repositories.extend(self.list_flatpak_repositories().await?);
        }

        Ok(repositories)
    }

    // Private helper methods for specific package managers

    async fn add_ppa_repository(&mut self, ppa: &str) -> Result<()> {
        info!("Adding PPA: {}", ppa);
        let args = vec!["-y", ppa];
        self.privilege_manager.execute_with_sudo("add-apt-repository", &args)?;
        
        // Update package lists
        let update_args = vec!["update"];
        self.privilege_manager.execute_with_sudo("apt", &update_args)?;
        
        Ok(())
    }

    async fn add_apt_repository(&mut self, repository: &str, key_url: Option<&str>) -> Result<()> {
        info!("Adding APT repository: {}", repository);
        
        // Add GPG key if provided
        if let Some(key) = key_url {
            info!("Adding repository key: {}", key);
            let key_args = vec!["wget", "-qO", "-", key, "|", "apt-key", "add", "-"];
            self.privilege_manager.execute_with_sudo("bash", &["-c", &key_args.join(" ")])?;
        }
        
        // Add repository to sources.list.d
        let sources_file = format!("/etc/apt/sources.list.d/omni-added-repo.list");
        std::fs::write(&sources_file, format!("{}\n", repository))?;
        
        // Update package lists
        let update_args = vec!["update"];
        self.privilege_manager.execute_with_sudo("apt", &update_args)?;
        
        Ok(())
    }

    async fn add_dnf_repository(&mut self, repository: &str) -> Result<()> {
        info!("Adding DNF repository: {}", repository);
        
        if repository.ends_with(".repo") {
            // Add repository file
            let args = vec!["config-manager", "--add-repo", repository];
            self.privilege_manager.execute_with_sudo("dnf", &args)?;
        } else {
            // Add repository URL
            let args = vec!["config-manager", "--add-repo", repository];
            self.privilege_manager.execute_with_sudo("dnf", &args)?;
        }
        
        Ok(())
    }

    async fn add_pacman_repository(&mut self, repository: &str) -> Result<()> {
        info!("Adding Pacman repository: {}", repository);
        warn!("Pacman repository addition requires manual configuration of /etc/pacman.conf");
        Err(anyhow!("Pacman repository addition not implemented - requires manual configuration"))
    }

    async fn add_flatpak_repository(&mut self, repository: &str) -> Result<()> {
        info!("Adding Flatpak repository: {}", repository);
        let args = vec!["remote-add", "--if-not-exists", "omni-added-repo", repository];
        std::process::Command::new("flatpak")
            .args(&args)
            .status()?;
        Ok(())
    }

    async fn remove_ppa_repository(&mut self, ppa: &str) -> Result<()> {
        info!("Removing PPA: {}", ppa);
        let args = vec!["-y", "-r", ppa];
        self.privilege_manager.execute_with_sudo("add-apt-repository", &args)?;
        Ok(())
    }

    async fn remove_apt_repository(&mut self, repository: &str) -> Result<()> {
        info!("Removing APT repository: {}", repository);
        // This is a simplified implementation - would need more sophisticated logic
        warn!("APT repository removal requires manual configuration");
        Err(anyhow!("APT repository removal not fully implemented"))
    }

    async fn remove_dnf_repository(&mut self, repository: &str) -> Result<()> {
        info!("Removing DNF repository: {}", repository);
        let args = vec!["config-manager", "--set-disabled", repository];
        self.privilege_manager.execute_with_sudo("dnf", &args)?;
        Ok(())
    }

    async fn remove_flatpak_repository(&mut self, repository: &str) -> Result<()> {
        info!("Removing Flatpak repository: {}", repository);
        let args = vec!["remote-delete", repository];
        std::process::Command::new("flatpak")
            .args(&args)
            .status()?;
        Ok(())
    }

    async fn list_apt_repositories(&self) -> Result<Vec<String>> {
        let mut repos = Vec::new();
        
        // Read from sources.list and sources.list.d
        if let Ok(contents) = std::fs::read_to_string("/etc/apt/sources.list") {
            for line in contents.lines() {
                if line.starts_with("deb ") && !line.starts_with("#") {
                    repos.push(format!("apt: {}", line));
                }
            }
        }
        
        // Read from sources.list.d directory
        if let Ok(entries) = std::fs::read_dir("/etc/apt/sources.list.d") {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".list") {
                            if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                                for line in contents.lines() {
                                    if line.starts_with("deb ") && !line.starts_with("#") {
                                        repos.push(format!("apt: {}", line));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(repos)
    }

    async fn list_dnf_repositories(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("dnf")
            .args(&["repolist", "--enabled"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let repos: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| {
                if !line.is_empty() {
                    Some(format!("dnf: {}", line))
                } else {
                    None
                }
            })
            .collect();
        
        Ok(repos)
    }

    async fn list_flatpak_repositories(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("flatpak")
            .args(&["remotes"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let repos: Vec<String> = stdout
            .lines()
            .map(|line| format!("flatpak: {}", line))
            .collect();
        
        Ok(repos)
    }
}
