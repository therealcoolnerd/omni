use crate::boxes::{apt, dnf, flatpak, pacman, snap, appimage};
use crate::distro;
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::snapshot::SnapshotManager;
use crate::manifest::OmniManifest;
use crate::input_validation::InputValidator;
use crate::privilege_manager::PrivilegeManager;
use crate::sandboxing::Sandbox;
use crate::history;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};

pub struct OmniBrain {
    mock_mode: bool,
    db: Option<Database>,
    snapshot_manager: Option<SnapshotManager>,
    privilege_manager: PrivilegeManager,
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
        }
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
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
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
    
    async fn install_with_specific_box(&self, app: &str, box_type: &str) -> Result<(String, String)> {
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
                
                Ok((box_type.to_string(), self.get_package_version(app, box_type).await?))
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
                
                Ok((box_type.to_string(), self.get_package_version(app, box_type).await?))
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
                
                Ok((box_type.to_string(), self.get_package_version(app, box_type).await?))
            }
            "snap" if distro::command_exists("snap") => {
                let args = vec!["install", app];
                if PrivilegeManager::is_root() {
                    sandbox.execute("snap", &args)?;
                } else {
                    self.privilege_manager.execute_with_sudo("snap", &args)?;
                }
                
                Ok((box_type.to_string(), self.get_package_version(app, box_type).await?))
            }
            "flatpak" if distro::command_exists("flatpak") => {
                let args = vec!["install", "-y", app];
                sandbox.execute("flatpak", &args)?;
                
                Ok((box_type.to_string(), self.get_package_version(app, box_type).await?))
            }
            _ => {
                Err(anyhow!("Box type '{}' not available or not supported", box_type))
            }
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
        let boxes = [
            ("apt", apt::install_with_apt as fn(&str)),
            ("dnf", dnf::install_with_dnf as fn(&str)),
            ("pacman", pacman::install_with_pacman as fn(&str)),
        ];
        
        for (box_name, install_fn) in &boxes {
            if distro::command_exists(box_name) {
                info!("Trying to install {} with {}", app, box_name);
                install_fn(app);
                return Ok((box_name.to_string(), "unknown".to_string()));
            }
        }
        
        // Try snap
        if distro::command_exists("snap") {
            info!("Trying to install {} with snap", app);
            snap::install_with_snap(app)?;
            return Ok(("snap".to_string(), "unknown".to_string()));
        }
        
        // Try flatpak
        if distro::command_exists("flatpak") {
            info!("Trying to install {} with flatpak", app);
            flatpak::install_with_flatpak(app);
            return Ok(("flatpak".to_string(), "unknown".to_string()));
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
            let _ = snapshot_manager.auto_snapshot("manifest", &manifest.project).await;
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
                    apt::install_with_apt(&app.name);
                    self.record_manifest_install(&app.name, "apt", app.source.as_deref()).await;
                    true
                }
                "pacman" if distro::command_exists("pacman") => {
                    pacman::install_with_pacman(&app.name);
                    self.record_manifest_install(&app.name, "pacman", app.source.as_deref()).await;
                    true
                }
                "dnf" if distro::command_exists("dnf") => {
                    dnf::install_with_dnf(&app.name);
                    self.record_manifest_install(&app.name, "dnf", app.source.as_deref()).await;
                    true
                }
                "flatpak" if distro::command_exists("flatpak") => {
                    let name = app.source.as_deref().unwrap_or(&app.name);
                    flatpak::install_with_flatpak(name);
                    self.record_manifest_install(&app.name, "flatpak", app.source.as_deref()).await;
                    true
                }
                "snap" if distro::command_exists("snap") => {
                    if let Ok(_) = snap::install_with_snap(&app.name) {
                        self.record_manifest_install(&app.name, "snap", app.source.as_deref()).await;
                        true
                    } else {
                        false
                    }
                }
                "appimage" => {
                    if let Some(url) = &app.source {
                        if let Ok(_) = appimage::install_appimage(url, &app.name).await {
                            self.record_manifest_install(&app.name, "appimage", app.source.as_deref()).await;
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
                            apt::install_with_apt(&app.name);
                            self.record_manifest_install(&app.name, "apt", None).await;
                        }
                        "pacman" if distro::command_exists("pacman") => {
                            pacman::install_with_pacman(&app.name);
                            self.record_manifest_install(&app.name, "pacman", None).await;
                        }
                        "dnf" if distro::command_exists("dnf") => {
                            dnf::install_with_dnf(&app.name);
                            self.record_manifest_install(&app.name, "dnf", None).await;
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
    
    async fn record_manifest_install(&self, package_name: &str, box_type: &str, source_url: Option<&str>) {
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
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
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
                snap::remove_snap(app)?;
                Ok(box_type.to_string())
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
            _ => {
                Err(anyhow::anyhow!("Box type '{}' not available or not supported", box_type))
            }
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
                        self.remove(&last_record.package_name, Some(&last_record.box_type)).await?;
                    }
                    InstallStatus::Removed => {
                        info!("Re-installing {}", last_record.package_name);
                        self.install(&last_record.package_name, Some(&last_record.box_type)).await?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Cannot undo operation with status: {:?}", last_record.status));
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
            let snapshot_id = snapshot_manager.create_snapshot(&snapshot_name, Some("Manual snapshot")).await?;
            println!("✅ Created snapshot '{}' with ID: {}", snapshot_name, snapshot_id);
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
                snapshot_manager.revert_to_snapshot(&latest_snapshot.id).await?;
                println!("✅ Reverted to snapshot '{}'", latest_snapshot.name);
            } else {
                return Err(anyhow::anyhow!("No snapshots available"));
            }
        }
        
        Ok(())
    }
}