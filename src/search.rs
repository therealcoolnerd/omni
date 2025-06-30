use crate::boxes::snap;
use crate::distro;
use crate::database::{Database, PackageCache};
use anyhow::Result;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::process::Command;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub box_type: String,
    pub source: Option<String>,
    pub installed: bool,
}

pub struct SearchEngine {
    db: Database,
}

impl SearchEngine {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        Ok(Self { db })
    }
    
    pub async fn search_all(&self, query: &str) -> Result<Vec<SearchResult>> {
        info!("Searching for: {}", query);
        
        let mut results = Vec::new();
        let installed_packages = self.get_installed_package_names().await?;
        
        // Search apt
        if distro::command_exists("apt") {
            if let Ok(apt_results) = self.search_apt(query).await {
                for mut result in apt_results {
                    result.installed = installed_packages.contains(&format!("{}:apt", result.name));
                    results.push(result);
                }
            }
        }
        
        // Search dnf
        if distro::command_exists("dnf") {
            if let Ok(dnf_results) = self.search_dnf(query).await {
                for mut result in dnf_results {
                    result.installed = installed_packages.contains(&format!("{}:dnf", result.name));
                    results.push(result);
                }
            }
        }
        
        // Search pacman
        if distro::command_exists("pacman") {
            if let Ok(pacman_results) = self.search_pacman(query).await {
                for mut result in pacman_results {
                    result.installed = installed_packages.contains(&format!("{}:pacman", result.name));
                    results.push(result);
                }
            }
        }
        
        // Search snap
        if distro::command_exists("snap") {
            if let Ok(snap_results) = self.search_snap(query).await {
                for mut result in snap_results {
                    result.installed = installed_packages.contains(&format!("{}:snap", result.name));
                    results.push(result);
                }
            }
        }
        
        // Search flatpak
        if distro::command_exists("flatpak") {
            if let Ok(flatpak_results) = self.search_flatpak(query).await {
                for mut result in flatpak_results {
                    result.installed = installed_packages.contains(&format!("{}:flatpak", result.name));
                    results.push(result);
                }
            }
        }
        
        // Deduplicate results by name, preferring installed packages
        let mut unique_results: HashMap<String, SearchResult> = HashMap::new();
        for result in results {
            let key = result.name.clone();
            if let Some(existing) = unique_results.get(&key) {
                if result.installed && !existing.installed {
                    unique_results.insert(key, result);
                }
            } else {
                unique_results.insert(key, result);
            }
        }
        
        let mut final_results: Vec<SearchResult> = unique_results.into_values().collect();
        final_results.sort_by(|a, b| {
            // Sort by installed status first, then by name
            match (a.installed, b.installed) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        info!("Found {} unique search results", final_results.len());
        Ok(final_results)
    }
    
    async fn get_installed_package_names(&self) -> Result<std::collections::HashSet<String>> {
        let installed = self.db.get_installed_packages().await?;
        Ok(installed
            .into_iter()
            .map(|p| format!("{}:{}", p.package_name, p.box_type))
            .collect())
    }
    
    async fn search_apt(&self, query: &str) -> Result<Vec<SearchResult>> {
        let output = Command::new("apt")
            .arg("search")
            .arg("--names-only")
            .arg(query)
            .output()?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        
        for line in stdout.lines() {
            if line.contains("/") && line.contains("-") {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() == 2 {
                    let name_version = parts[0];
                    let description = parts[1];
                    
                    if let Some(name) = name_version.split('/').next() {
                        results.push(SearchResult {
                            name: name.to_string(),
                            description: Some(description.to_string()),
                            version: None,
                            box_type: "apt".to_string(),
                            source: None,
                            installed: false,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn search_dnf(&self, query: &str) -> Result<Vec<SearchResult>> {
        let output = Command::new("dnf")
            .arg("search")
            .arg("--quiet")
            .arg(query)
            .output()?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        
        for line in stdout.lines() {
            if line.contains(".") && line.contains(":") {
                let parts: Vec<&str> = line.splitn(2, " : ").collect();
                if parts.len() == 2 {
                    let name_arch = parts[0];
                    let description = parts[1];
                    
                    if let Some(name) = name_arch.split('.').next() {
                        results.push(SearchResult {
                            name: name.to_string(),
                            description: Some(description.to_string()),
                            version: None,
                            box_type: "dnf".to_string(),
                            source: None,
                            installed: false,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn search_pacman(&self, query: &str) -> Result<Vec<SearchResult>> {
        let output = Command::new("pacman")
            .arg("-Ss")
            .arg(query)
            .output()?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if line.starts_with("extra/") || line.starts_with("core/") || line.starts_with("community/") {
                let parts: Vec<&str> = line.splitn(2, " ").collect();
                if parts.len() == 2 {
                    let repo_name = parts[0];
                    let version = parts[1];
                    
                    if let Some(name) = repo_name.split('/').nth(1) {
                        let description = if i + 1 < lines.len() {
                            Some(lines[i + 1].trim().to_string())
                        } else {
                            None
                        };
                        
                        results.push(SearchResult {
                            name: name.to_string(),
                            description,
                            version: Some(version.to_string()),
                            box_type: "pacman".to_string(),
                            source: None,
                            installed: false,
                        });
                        
                        i += 2; // Skip description line
                        continue;
                    }
                }
            }
            i += 1;
        }
        
        Ok(results)
    }
    
    async fn search_snap(&self, query: &str) -> Result<Vec<SearchResult>> {
        match snap::search_snap(query) {
            Ok(packages) => {
                let mut results = Vec::new();
                for package in packages {
                    results.push(SearchResult {
                        name: package,
                        description: None,
                        version: None,
                        box_type: "snap".to_string(),
                        source: None,
                        installed: false,
                    });
                }
                Ok(results)
            }
            Err(_) => Ok(vec![]),
        }
    }
    
    async fn search_flatpak(&self, query: &str) -> Result<Vec<SearchResult>> {
        let output = Command::new("flatpak")
            .arg("search")
            .arg(query)
            .output()?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let description = parts[1];
                let app_id = parts[2];
                
                results.push(SearchResult {
                    name: name.to_string(),
                    description: Some(description.to_string()),
                    version: None,
                    box_type: "flatpak".to_string(),
                    source: Some(app_id.to_string()),
                    installed: false,
                });
            }
        }
        
        Ok(results)
    }
    
    pub async fn get_package_info(&self, package_name: &str, box_type: &str) -> Result<Option<String>> {
        // Try to get from cache first
        if let Ok(Some(cached)) = self.db.get_cached_package_info(package_name, box_type).await {
            // Check if cache is still fresh (1 hour)
            let age = Utc::now().signed_duration_since(cached.cached_at);
            if age.num_hours() < 1 {
                return Ok(Some(format!(
                    "Package: {}\nVersion: {}\nDescription: {}\nDependencies: {}",
                    cached.package_name,
                    cached.version,
                    cached.description.unwrap_or("Not available".to_string()),
                    cached.dependencies.join(", ")
                )));
            }
        }
        
        // Get fresh info from package manager
        let info = match box_type {
            "apt" => self.get_apt_info(package_name).await,
            "dnf" => self.get_dnf_info(package_name).await,
            "pacman" => self.get_pacman_info(package_name).await,
            "snap" => snap::get_snap_info(package_name).map_err(|e| anyhow::anyhow!(e)),
            "flatpak" => self.get_flatpak_info(package_name).await,
            _ => Err(anyhow::anyhow!("Unsupported box type")),
        };
        
        match info {
            Ok(info_text) => {
                // Cache the result for future use
                let cache_entry = PackageCache {
                    package_name: package_name.to_string(),
                    box_type: box_type.to_string(),
                    version: "unknown".to_string(),
                    description: Some("Package info".to_string()),
                    dependencies: vec![],
                    cached_at: Utc::now(),
                };
                
                let _ = self.db.cache_package_info(&cache_entry).await;
                Ok(Some(info_text))
            }
            Err(e) => {
                warn!("Failed to get package info: {}", e);
                Ok(None)
            }
        }
    }
    
    async fn get_apt_info(&self, package_name: &str) -> Result<String> {
        let output = Command::new("apt")
            .arg("show")
            .arg(package_name)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Package not found"))
        }
    }
    
    async fn get_dnf_info(&self, package_name: &str) -> Result<String> {
        let output = Command::new("dnf")
            .arg("info")
            .arg(package_name)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Package not found"))
        }
    }
    
    async fn get_pacman_info(&self, package_name: &str) -> Result<String> {
        let output = Command::new("pacman")
            .arg("-Si")
            .arg(package_name)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Package not found"))
        }
    }
    
    async fn get_flatpak_info(&self, package_name: &str) -> Result<String> {
        let output = Command::new("flatpak")
            .arg("info")
            .arg(package_name)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Package not found"))
        }
    }
}