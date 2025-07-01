use crate::database::Database;
use crate::distro;
use crate::search::SearchEngine;
use anyhow::Result;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::process::Command;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: Option<String>,
    pub box_type: String,
    pub optional: bool,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub box_type: String,
    pub dependencies: Vec<Dependency>,
    pub source_url: Option<String>,
    pub install_order: usize,
}

#[derive(Debug)]
pub struct ResolutionPlan {
    pub packages: Vec<ResolvedPackage>,
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
    pub total_size: Option<u64>,
}

pub struct DependencyResolver {
    db: Database,
    search_engine: SearchEngine,
}

impl DependencyResolver {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        let search_engine = SearchEngine::new().await?;

        Ok(Self { db, search_engine })
    }

    pub async fn resolve_dependencies(
        &self,
        package_name: &str,
        box_type: Option<&str>,
    ) -> Result<ResolutionPlan> {
        info!("Resolving dependencies for package: {}", package_name);

        let mut resolution_plan = ResolutionPlan {
            packages: Vec::new(),
            conflicts: Vec::new(),
            warnings: Vec::new(),
            total_size: None,
        };

        let mut visited = HashSet::new();
        let mut resolved = HashMap::new();
        let mut queue = VecDeque::new();

        // Start with the requested package
        queue.push_back((package_name.to_string(), box_type.map(|s| s.to_string()), 0));

        while let Some((pkg_name, preferred_box, depth)) = queue.pop_front() {
            if visited.contains(&pkg_name) {
                continue;
            }

            visited.insert(pkg_name.clone());

            // Get package dependencies
            let dependencies = self
                .get_package_dependencies(&pkg_name, preferred_box.as_deref())
                .await?;

            // Check for conflicts
            let conflicts = self.check_conflicts(&pkg_name, &dependencies).await?;
            if !conflicts.is_empty() {
                resolution_plan.conflicts.extend(conflicts);
            }

            // Determine best box type if not specified
            let selected_box = if let Some(box_type) = &preferred_box {
                box_type.clone()
            } else {
                self.select_best_box(&pkg_name).await?
            };

            // Get package version
            let version = self.get_package_version(&pkg_name, &selected_box).await?;

            let resolved_package = ResolvedPackage {
                name: pkg_name.clone(),
                version,
                box_type: selected_box,
                dependencies: dependencies.clone(),
                source_url: None,
                install_order: depth,
            };

            resolved.insert(pkg_name.clone(), resolved_package);

            // Add dependencies to queue
            for dep in dependencies {
                if !dep.optional && !visited.contains(&dep.name) {
                    queue.push_back((dep.name, Some(dep.box_type), depth + 1));
                }
            }
        }

        // Sort packages by install order (dependencies first)
        let mut packages: Vec<ResolvedPackage> = resolved.into_values().collect();
        packages.sort_by_key(|p| std::cmp::Reverse(p.install_order));

        resolution_plan.packages = packages;

        // Calculate total download size if possible
        resolution_plan.total_size = self.calculate_total_size(&resolution_plan.packages).await;

        info!(
            "Dependency resolution complete. {} packages to install",
            resolution_plan.packages.len()
        );

        Ok(resolution_plan)
    }

    async fn get_package_dependencies(
        &self,
        package_name: &str,
        box_type: Option<&str>,
    ) -> Result<Vec<Dependency>> {
        // Try to get from cache first
        if let Some(box_type) = box_type {
            if let Ok(Some(cached)) = self
                .db
                .get_cached_package_info(package_name, box_type)
                .await
            {
                return Ok(cached
                    .dependencies
                    .into_iter()
                    .map(|name| Dependency {
                        name,
                        version_req: None,
                        box_type: box_type.to_string(),
                        optional: false,
                        conflicts: vec![],
                        provides: vec![],
                    })
                    .collect());
            }
        }

        // Get fresh dependency information
        let dependencies = match box_type {
            Some("apt") => self.get_apt_dependencies(package_name).await,
            Some("dnf") => self.get_dnf_dependencies(package_name).await,
            Some("pacman") => self.get_pacman_dependencies(package_name).await,
            Some("snap") => self.get_snap_dependencies(package_name).await,
            Some("flatpak") => self.get_flatpak_dependencies(package_name).await,
            _ => {
                // Try all available package managers
                self.get_dependencies_from_available_boxes(package_name)
                    .await
            }
        };

        Ok(dependencies.unwrap_or_else(|_| vec![]))
    }

    async fn get_apt_dependencies(&self, package_name: &str) -> Result<Vec<Dependency>> {
        if !distro::command_exists("apt") {
            return Ok(vec![]);
        }

        let output = Command::new("apt")
            .arg("depends")
            .arg(package_name)
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dependencies = Vec::new();

        for line in stdout.lines() {
            if line.trim().starts_with("Depends:") {
                let dep_part = line.trim().strip_prefix("Depends:").unwrap_or("").trim();
                if !dep_part.is_empty() {
                    dependencies.push(Dependency {
                        name: dep_part.split_whitespace().next().unwrap_or("").to_string(),
                        version_req: None,
                        box_type: "apt".to_string(),
                        optional: false,
                        conflicts: vec![],
                        provides: vec![],
                    });
                }
            } else if line.trim().starts_with("Recommends:") {
                let dep_part = line.trim().strip_prefix("Recommends:").unwrap_or("").trim();
                if !dep_part.is_empty() {
                    dependencies.push(Dependency {
                        name: dep_part.split_whitespace().next().unwrap_or("").to_string(),
                        version_req: None,
                        box_type: "apt".to_string(),
                        optional: true,
                        conflicts: vec![],
                        provides: vec![],
                    });
                }
            } else if line.trim().starts_with("Conflicts:") {
                let conflict_part = line.trim().strip_prefix("Conflicts:").unwrap_or("").trim();
                if !conflict_part.is_empty() {
                    // Add conflict information to the last dependency
                    if let Some(last_dep) = dependencies.last_mut() {
                        last_dep.conflicts.push(
                            conflict_part
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .to_string(),
                        );
                    }
                }
            }
        }

        Ok(dependencies)
    }

    async fn get_dnf_dependencies(&self, package_name: &str) -> Result<Vec<Dependency>> {
        if !distro::command_exists("dnf") {
            return Ok(vec![]);
        }

        let output = Command::new("dnf")
            .arg("repoquery")
            .arg("--requires")
            .arg(package_name)
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dependencies = Vec::new();

        for line in stdout.lines() {
            let dep_name = line.trim();
            if !dep_name.is_empty() && !dep_name.starts_with('/') {
                // Filter out file dependencies and system dependencies
                if !dep_name.contains("(") && !dep_name.starts_with("rpmlib") {
                    dependencies.push(Dependency {
                        name: dep_name.to_string(),
                        version_req: None,
                        box_type: "dnf".to_string(),
                        optional: false,
                        conflicts: vec![],
                        provides: vec![],
                    });
                }
            }
        }

        Ok(dependencies)
    }

    async fn get_pacman_dependencies(&self, package_name: &str) -> Result<Vec<Dependency>> {
        if !distro::command_exists("pacman") {
            return Ok(vec![]);
        }

        let output = Command::new("pacman")
            .arg("-Si")
            .arg(package_name)
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dependencies = Vec::new();
        for line in stdout.lines() {
            if line.starts_with("Depends On") {
                let deps_part = line.split(':').nth(1).unwrap_or("").trim();
                if deps_part != "None" {
                    for dep in deps_part.split_whitespace() {
                        dependencies.push(Dependency {
                            name: dep.to_string(),
                            version_req: None,
                            box_type: "pacman".to_string(),
                            optional: false,
                            conflicts: vec![],
                            provides: vec![],
                        });
                    }
                }
            } else if line.starts_with("Optional Deps") {
                let deps_part = line.split(':').nth(1).unwrap_or("").trim();
                if deps_part != "None" {
                    for dep in deps_part.split_whitespace() {
                        dependencies.push(Dependency {
                            name: dep.split(':').next().unwrap_or("").to_string(),
                            version_req: None,
                            box_type: "pacman".to_string(),
                            optional: true,
                            conflicts: vec![],
                            provides: vec![],
                        });
                    }
                }
                break; // Optional deps is usually the last relevant section
            }
        }

        Ok(dependencies)
    }

    async fn get_snap_dependencies(&self, package_name: &str) -> Result<Vec<Dependency>> {
        // Snaps are generally self-contained with minimal external dependencies
        Ok(vec![])
    }

    async fn get_flatpak_dependencies(&self, package_name: &str) -> Result<Vec<Dependency>> {
        if !distro::command_exists("flatpak") {
            return Ok(vec![]);
        }

        let output = Command::new("flatpak")
            .arg("info")
            .arg("--show-runtime")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let runtime = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !runtime.is_empty() {
                return Ok(vec![Dependency {
                    name: runtime,
                    version_req: None,
                    box_type: "flatpak".to_string(),
                    optional: false,
                    conflicts: vec![],
                    provides: vec![],
                }]);
            }
        }

        Ok(vec![])
    }

    async fn get_dependencies_from_available_boxes(
        &self,
        package_name: &str,
    ) -> Result<Vec<Dependency>> {
        let mut all_dependencies = Vec::new();

        // Try each available package manager
        if let Ok(deps) = self.get_apt_dependencies(package_name).await {
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.get_dnf_dependencies(package_name).await {
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.get_pacman_dependencies(package_name).await {
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.get_flatpak_dependencies(package_name).await {
            all_dependencies.extend(deps);
        }

        // Deduplicate by name
        let mut unique_deps = HashMap::new();
        for dep in all_dependencies {
            unique_deps.entry(dep.name.clone()).or_insert(dep);
        }

        Ok(unique_deps.into_values().collect())
    }

    async fn check_conflicts(
        &self,
        package_name: &str,
        dependencies: &[Dependency],
    ) -> Result<Vec<String>> {
        let mut conflicts = Vec::new();

        // Get currently installed packages
        let installed = self.db.get_installed_packages().await?;
        let installed_names: HashSet<String> =
            installed.iter().map(|p| p.package_name.clone()).collect();

        // Check for explicit conflicts
        for dep in dependencies {
            for conflict in &dep.conflicts {
                if installed_names.contains(conflict) {
                    conflicts.push(format!(
                        "Package {} conflicts with installed package {}",
                        package_name, conflict
                    ));
                }
            }
        }

        Ok(conflicts)
    }

    async fn select_best_box(&self, package_name: &str) -> Result<String> {
        // Search across all available package managers and select the best one
        let search_results = self.search_engine.search_all(package_name).await?;

        // Prefer exact name matches
        let exact_matches: Vec<_> = search_results
            .iter()
            .filter(|r| r.name == package_name)
            .collect();

        if !exact_matches.is_empty() {
            // Priority order: apt > dnf > pacman > flatpak > snap > appimage
            let priority_order = ["apt", "dnf", "pacman", "flatpak", "snap", "appimage"];

            for box_type in &priority_order {
                if let Some(result) = exact_matches.iter().find(|r| &r.box_type == box_type) {
                    return Ok(result.box_type.clone());
                }
            }

            // If no priority match, return the first exact match
            return Ok(exact_matches[0].box_type.clone());
        }

        // Fallback to detecting distro default
        Ok(distro::detect_distro())
    }

    async fn get_package_version(&self, package_name: &str, box_type: &str) -> Result<String> {
        let version = match box_type {
            "apt" => self.get_apt_version(package_name).await,
            "dnf" => self.get_dnf_version(package_name).await,
            "pacman" => self.get_pacman_version(package_name).await,
            "snap" => self.get_snap_version(package_name).await,
            "flatpak" => self.get_flatpak_version(package_name).await,
            _ => Ok("unknown".to_string()),
        };

        Ok(version.unwrap_or_else(|_| "unknown".to_string()))
    }

    async fn get_apt_version(&self, package_name: &str) -> Result<String> {
        let output = Command::new("apt")
            .arg("policy")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("Candidate:") {
                    let version = line.split(':').nth(1).unwrap_or("").trim();
                    if version != "(none)" {
                        return Ok(version.to_string());
                    }
                }
            }
        }

        Ok("unknown".to_string())
    }

    async fn get_dnf_version(&self, package_name: &str) -> Result<String> {
        let output = Command::new("dnf").arg("info").arg(package_name).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Version") {
                    let version = line.split(':').nth(1).unwrap_or("").trim();
                    return Ok(version.to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    async fn get_pacman_version(&self, package_name: &str) -> Result<String> {
        let output = Command::new("pacman")
            .arg("-Si")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Version") {
                    let version = line.split(':').nth(1).unwrap_or("").trim();
                    return Ok(version.to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    async fn get_snap_version(&self, package_name: &str) -> Result<String> {
        let output = Command::new("snap")
            .arg("info")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("tracking:") {
                    let version = line.split_whitespace().nth(1).unwrap_or("").trim();
                    return Ok(version.to_string());
                }
            }
        }

        Ok("latest".to_string())
    }

    async fn get_flatpak_version(&self, package_name: &str) -> Result<String> {
        let output = Command::new("flatpak")
            .arg("info")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Version:") {
                    let version = line.split(':').nth(1).unwrap_or("").trim();
                    return Ok(version.to_string());
                }
            }
        }

        Ok("latest".to_string())
    }

    async fn calculate_total_size(&self, packages: &[ResolvedPackage]) -> Option<u64> {
        let mut total_size = 0u64;
        let mut found_any_size = false;

        for package in packages {
            if let Ok(size) = self
                .get_package_size(&package.name, &package.box_type)
                .await
            {
                total_size += size;
                found_any_size = true;
            }
        }

        if found_any_size {
            Some(total_size)
        } else {
            None
        }
    }

    async fn get_package_size(&self, package_name: &str, box_type: &str) -> Result<u64> {
        match box_type {
            "apt" => self.get_apt_size(package_name).await,
            "dnf" => self.get_dnf_size(package_name).await,
            "pacman" => self.get_pacman_size(package_name).await,
            _ => Ok(0), // Size information not available for other package types
        }
    }

    async fn get_apt_size(&self, package_name: &str) -> Result<u64> {
        let output = Command::new("apt").arg("show").arg(package_name).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Size:") {
                    let size_str = line.split(':').nth(1).unwrap_or("").trim();
                    if let Ok(size) = size_str.parse::<u64>() {
                        return Ok(size);
                    }
                }
            }
        }

        Ok(0)
    }

    async fn get_dnf_size(&self, package_name: &str) -> Result<u64> {
        let output = Command::new("dnf").arg("info").arg(package_name).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Size") {
                    let size_str = line.split(':').nth(1).unwrap_or("").trim();
                    // Parse size string like "1.2 M" or "500 k"
                    let parts: Vec<&str> = size_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(num) = parts[0].parse::<f64>() {
                            let multiplier = match parts[1].to_lowercase().as_str() {
                                "k" => 1024,
                                "m" => 1024 * 1024,
                                "g" => 1024 * 1024 * 1024,
                                _ => 1,
                            };
                            return Ok((num * multiplier as f64) as u64);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn get_pacman_size(&self, package_name: &str) -> Result<u64> {
        let output = Command::new("pacman")
            .arg("-Si")
            .arg(package_name)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Download Size") {
                    let size_str = line.split(':').nth(1).unwrap_or("").trim();
                    // Parse size string like "1.20 MiB"
                    let parts: Vec<&str> = size_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(num) = parts[0].parse::<f64>() {
                            let multiplier = match parts[1].to_lowercase().as_str() {
                                "kib" => 1024,
                                "mib" => 1024 * 1024,
                                "gib" => 1024 * 1024 * 1024,
                                _ => 1,
                            };
                            return Ok((num * multiplier as f64) as u64);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}
