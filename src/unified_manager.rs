use crate::boxes::{
    apt::AptManager, brew::BrewBox, chocolatey::ChocolateyBox, dnf::DnfBox, emerge::EmergeBox,
    flatpak::FlatpakBox, mas::MasBox, nix::NixBox, pacman::PacmanBox, scoop::ScoopBox,
    snap::SnapBox, winget::WingetBox, zypper::ZypperBox,
};
use crate::config::OmniConfig;
use crate::distro::PackageManager;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{error, info, warn};

pub struct UnifiedPackageManager {
    config: OmniConfig,
    managers: HashMap<String, Box<dyn PackageManager>>,
}

impl UnifiedPackageManager {
    pub fn new(config: OmniConfig) -> Result<Self> {
        let mut managers: HashMap<String, Box<dyn PackageManager>> = HashMap::new();

        // Initialize all available package managers
        if AptManager::is_available() {
            managers.insert("apt".to_string(), Box::new(AptManager::new()?));
        }

        if DnfBox::is_available() {
            managers.insert("dnf".to_string(), Box::new(DnfBox::new()?));
        }

        if PacmanBox::is_available() {
            managers.insert("pacman".to_string(), Box::new(PacmanBox::new()?));
        }

        if SnapBox::is_available() && config.is_box_enabled("snap") {
            managers.insert("snap".to_string(), Box::new(SnapBox::new()?));
        }

        if FlatpakBox::is_available() && config.is_box_enabled("flatpak") {
            managers.insert("flatpak".to_string(), Box::new(FlatpakBox::new()?));
        }

        if ChocolateyBox::is_available() {
            managers.insert("chocolatey".to_string(), Box::new(ChocolateyBox::new()?));
        }

        if BrewBox::is_available() {
            managers.insert("brew".to_string(), Box::new(BrewBox::new()?));
        }

        if WingetBox::is_available() {
            managers.insert("winget".to_string(), Box::new(WingetBox::new()?));
        }

        if ScoopBox::is_available() {
            managers.insert("scoop".to_string(), Box::new(ScoopBox::new()?));
        }

        if MasBox::is_available() {
            managers.insert("mas".to_string(), Box::new(MasBox::new()?));
        }

        if ZypperBox::is_available() {
            managers.insert("zypper".to_string(), Box::new(ZypperBox::new()?));
        }

        if EmergeBox::is_available() {
            managers.insert("emerge".to_string(), Box::new(EmergeBox::new()?));
        }

        if NixBox::is_available() {
            managers.insert("nix".to_string(), Box::new(NixBox::new()?));
        }

        info!("Initialized {} package managers", managers.len());
        for manager_name in managers.keys() {
            info!("  - {}", manager_name);
        }

        Ok(Self { config, managers })
    }

    pub fn install(&self, package: &str) -> Result<String> {
        self.install_with_box(package, None)
    }

    pub fn install_with_box(&self, package: &str, preferred_box: Option<&str>) -> Result<String> {
        let box_order = if let Some(box_name) = preferred_box {
            vec![box_name.to_string()]
        } else {
            self.get_preferred_box_order()
        };

        let mut last_error = None;

        for box_name in &box_order {
            if !self.config.is_box_enabled(box_name) {
                continue;
            }

            if let Some(manager) = self.managers.get(box_name) {
                info!("Attempting to install '{}' with {}", package, box_name);

                match manager.install(package) {
                    Ok(()) => {
                        info!("✅ Successfully installed '{}' with {}", package, box_name);
                        return Ok(box_name.clone());
                    }
                    Err(e) => {
                        warn!(
                            "❌ Failed to install '{}' with {}: {}",
                            package, box_name, e
                        );
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        if let Some(err) = last_error {
            Err(anyhow!(
                "Failed to install '{}' with any available package manager: {}",
                package,
                err
            ))
        } else {
            Err(anyhow!(
                "No suitable package managers available for '{}'",
                package
            ))
        }
    }

    pub fn remove(&self, package: &str) -> Result<String> {
        self.remove_with_box(package, None)
    }

    pub fn remove_with_box(&self, package: &str, preferred_box: Option<&str>) -> Result<String> {
        let box_order = if let Some(box_name) = preferred_box {
            vec![box_name.to_string()]
        } else {
            self.get_preferred_box_order()
        };

        let mut last_error = None;

        for box_name in &box_order {
            if !self.config.is_box_enabled(box_name) {
                continue;
            }

            if let Some(manager) = self.managers.get(box_name) {
                info!("Attempting to remove '{}' with {}", package, box_name);

                match manager.remove(package) {
                    Ok(()) => {
                        info!("✅ Successfully removed '{}' with {}", package, box_name);
                        return Ok(box_name.clone());
                    }
                    Err(e) => {
                        warn!("❌ Failed to remove '{}' with {}: {}", package, box_name, e);
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        if let Some(err) = last_error {
            Err(anyhow!(
                "Failed to remove '{}' with any available package manager: {}",
                package,
                err
            ))
        } else {
            Err(anyhow!(
                "No suitable package managers available for '{}'",
                package
            ))
        }
    }

    pub fn update(&self, package: Option<&str>) -> Result<()> {
        let box_order = self.get_preferred_box_order();
        let mut updated_any = false;

        for box_name in &box_order {
            if !self.config.is_box_enabled(box_name) {
                continue;
            }

            if let Some(manager) = self.managers.get(box_name) {
                info!("Updating packages with {}", box_name);

                match manager.update(package) {
                    Ok(()) => {
                        info!("✅ Successfully updated packages with {}", box_name);
                        updated_any = true;
                    }
                    Err(e) => {
                        warn!("❌ Failed to update packages with {}: {}", box_name, e);
                    }
                }
            }
        }

        if updated_any {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to update packages with any available package manager"
            ))
        }
    }

    pub fn search(&self, query: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();
        let box_order = self.get_preferred_box_order();

        for box_name in &box_order {
            if !self.config.is_box_enabled(box_name) {
                continue;
            }

            if let Some(manager) = self.managers.get(box_name) {
                match manager.search(query) {
                    Ok(packages) => {
                        if !packages.is_empty() {
                            results.insert(box_name.clone(), packages);
                        }
                    }
                    Err(e) => {
                        warn!("❌ Search failed with {}: {}", box_name, e);
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn list_installed(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();

        for (box_name, manager) in &self.managers {
            if !self.config.is_box_enabled(box_name) {
                continue;
            }

            match manager.list_installed() {
                Ok(packages) => {
                    if !packages.is_empty() {
                        results.insert(box_name.clone(), packages);
                    }
                }
                Err(e) => {
                    warn!(
                        "❌ Failed to list installed packages with {}: {}",
                        box_name, e
                    );
                }
            }
        }

        Ok(results)
    }

    pub fn get_info(&self, package: &str, box_name: &str) -> Result<String> {
        if let Some(manager) = self.managers.get(box_name) {
            manager.get_info(package)
        } else {
            Err(anyhow!("Package manager '{}' not available", box_name))
        }
    }

    fn get_preferred_box_order(&self) -> Vec<String> {
        let mut order = Vec::new();

        // Start with user's preferred order
        for box_name in &self.config.boxes.preferred_order {
            if self.managers.contains_key(box_name) {
                order.push(box_name.clone());
            }
        }

        // Add any remaining available managers
        for box_name in self.managers.keys() {
            if !order.contains(box_name) {
                order.push(box_name.clone());
            }
        }

        order
    }

    pub fn get_available_managers(&self) -> Vec<String> {
        self.managers.keys().cloned().collect()
    }

    pub fn reload_config(&mut self, config: OmniConfig) -> Result<()> {
        info!("Reloading configuration");
        self.config = config;
        Ok(())
    }

    pub fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        // Try to get version from the first package manager that has the package installed
        for box_name in &self.get_preferred_box_order() {
            if let Some(manager) = self.managers.get(box_name) {
                match manager.get_installed_version(package) {
                    Ok(Some(version)) => {
                        info!(
                            "✅ Found version '{}' for package '{}' from {}",
                            version, package, box_name
                        );
                        return Ok(Some(version));
                    }
                    Ok(None) => {
                        // Package not installed with this manager, try next one
                        continue;
                    }
                    Err(e) => {
                        warn!(
                            "❌ Error checking version for '{}' with {}: {}",
                            package, box_name, e
                        );
                        continue;
                    }
                }
            }
        }

        info!("ℹ️ Package '{}' not found in any package manager", package);
        Ok(None)
    }
}
