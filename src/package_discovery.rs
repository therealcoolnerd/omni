/// Package discovery service client for enhanced search and metadata
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const API_BASE_URL: &str = "https://therealcoolnerd.github.io/omni-packages/api/v1";
const CACHE_DURATION: Duration = Duration::from_secs(3600); // 1 hour cache

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub cross_platform: Option<CrossPlatformMappings>,
    pub popularity: Option<PopularityInfo>,
    pub security: Option<SecurityInfo>,
    pub similar_packages: Vec<String>,
    pub alternatives: Vec<AlternativePackage>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformMappings {
    pub linux: Option<HashMap<String, Vec<String>>>,
    pub macos: Option<HashMap<String, Vec<String>>>,
    pub windows: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityInfo {
    pub rank: Option<u32>,
    pub downloads_per_month: Option<u64>,
    pub github_stars: Option<u32>,
    pub search_frequency: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub score: Option<f32>,
    pub last_audit: Option<String>,
    pub vulnerabilities: Vec<String>,
    pub cve_count: u32,
    pub security_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativePackage {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularPackagesResponse {
    pub version: String,
    pub last_updated: String,
    pub total_packages: u32,
    pub popular_packages: Vec<PopularPackage>,
    pub categories: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularPackage {
    pub rank: u32,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub downloads_per_month: u64,
    pub search_frequency: u8,
    pub cross_platform: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformResponse {
    pub version: String,
    pub last_updated: String,
    pub mappings: HashMap<String, CrossPlatformMappings>,
    pub reverse_mappings: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    pub version: String,
    pub last_updated: String,
    pub security_scores: HashMap<String, SecurityInfo>,
    pub security_categories: HashMap<String, Vec<String>>,
    pub vulnerability_alerts: Vec<String>,
}

pub struct PackageDiscoveryService {
    client: Client,
    cache: RwLock<HashMap<String, (PackageMetadata, std::time::Instant)>>,
    popular_cache: RwLock<Option<(PopularPackagesResponse, std::time::Instant)>>,
    mappings_cache: RwLock<Option<(CrossPlatformResponse, std::time::Instant)>>,
    security_cache: RwLock<Option<(SecurityResponse, std::time::Instant)>>,
}

impl PackageDiscoveryService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Omni Package Manager")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            cache: RwLock::new(HashMap::new()),
            popular_cache: RwLock::new(None),
            mappings_cache: RwLock::new(None),
            security_cache: RwLock::new(None),
        }
    }

    /// Get package metadata with caching
    pub async fn get_package_metadata(&self, package_name: &str) -> Option<PackageMetadata> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((metadata, timestamp)) = cache.get(package_name) {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Some(metadata.clone());
                }
            }
        }

        // Fetch from API
        match self.fetch_package_metadata(package_name).await {
            Ok(metadata) => {
                // Update cache
                let mut cache = self.cache.write().await;
                cache.insert(package_name.to_string(), (metadata.clone(), std::time::Instant::now()));
                Some(metadata)
            }
            Err(e) => {
                warn!("Failed to fetch metadata for {}: {}", package_name, e);
                None
            }
        }
    }

    async fn fetch_package_metadata(&self, package_name: &str) -> Result<PackageMetadata> {
        let url = format!("{}/packages/all.json", API_BASE_URL);
        let response: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
        if let Some(packages) = response.get("packages").and_then(|p| p.as_array()) {
            for package in packages {
                if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
                    if name == package_name {
                        return Ok(serde_json::from_value(package.clone())?);
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Package not found: {}", package_name))
    }

    /// Get popular packages with caching
    pub async fn get_popular_packages(&self) -> Option<PopularPackagesResponse> {
        // Check cache first
        {
            let cache = self.popular_cache.read().await;
            if let Some((data, timestamp)) = cache.as_ref() {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Some(data.clone());
                }
            }
        }

        // Fetch from API
        match self.fetch_popular_packages().await {
            Ok(data) => {
                // Update cache
                let mut cache = self.popular_cache.write().await;
                *cache = Some((data.clone(), std::time::Instant::now()));
                Some(data)
            }
            Err(e) => {
                warn!("Failed to fetch popular packages: {}", e);
                None
            }
        }
    }

    async fn fetch_popular_packages(&self) -> Result<PopularPackagesResponse> {
        let url = format!("{}/packages/popular.json", API_BASE_URL);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    /// Get cross-platform mappings with caching
    pub async fn get_cross_platform_mappings(&self) -> Option<CrossPlatformResponse> {
        // Check cache first
        {
            let cache = self.mappings_cache.read().await;
            if let Some((data, timestamp)) = cache.as_ref() {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Some(data.clone());
                }
            }
        }

        // Fetch from API
        match self.fetch_cross_platform_mappings().await {
            Ok(data) => {
                // Update cache
                let mut cache = self.mappings_cache.write().await;
                *cache = Some((data.clone(), std::time::Instant::now()));
                Some(data)
            }
            Err(e) => {
                warn!("Failed to fetch cross-platform mappings: {}", e);
                None
            }
        }
    }

    async fn fetch_cross_platform_mappings(&self) -> Result<CrossPlatformResponse> {
        let url = format!("{}/packages/cross-platform.json", API_BASE_URL);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    /// Get security information with caching
    pub async fn get_security_info(&self) -> Option<SecurityResponse> {
        // Check cache first
        {
            let cache = self.security_cache.read().await;
            if let Some((data, timestamp)) = cache.as_ref() {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Some(data.clone());
                }
            }
        }

        // Fetch from API
        match self.fetch_security_info().await {
            Ok(data) => {
                // Update cache
                let mut cache = self.security_cache.write().await;
                *cache = Some((data.clone(), std::time::Instant::now()));
                Some(data)
            }
            Err(e) => {
                warn!("Failed to fetch security info: {}", e);
                None
            }
        }
    }

    async fn fetch_security_info(&self) -> Result<SecurityResponse> {
        let url = format!("{}/packages/security.json", API_BASE_URL);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    /// Find cross-platform equivalent package name
    pub async fn find_platform_package(&self, package_name: &str, target_platform: &str, target_manager: &str) -> Option<String> {
        let mappings = self.get_cross_platform_mappings().await?;
        
        // First, try direct mapping
        if let Some(platform_mappings) = mappings.mappings.get(package_name) {
            if let Some(platform_data) = match target_platform {
                "linux" => platform_mappings.linux.as_ref(),
                "macos" => platform_mappings.macos.as_ref(),
                "windows" => platform_mappings.windows.as_ref(),
                _ => None,
            } {
                if let Some(packages) = platform_data.get(target_manager) {
                    return packages.first().cloned();
                }
            }
        }

        // Try reverse mapping
        if let Some(platform_data) = mappings.reverse_mappings.get(target_platform) {
            if let Some(manager_data) = platform_data.get(target_manager) {
                if let Some(canonical_name) = manager_data.get(package_name) {
                    return Some(canonical_name.clone());
                }
            }
        }

        None
    }

    /// Get similar packages for discovery
    pub async fn get_similar_packages(&self, package_name: &str) -> Vec<String> {
        if let Some(metadata) = self.get_package_metadata(package_name).await {
            return metadata.similar_packages;
        }
        Vec::new()
    }

    /// Get package alternatives with reasons
    pub async fn get_package_alternatives(&self, package_name: &str) -> Vec<AlternativePackage> {
        if let Some(metadata) = self.get_package_metadata(package_name).await {
            return metadata.alternatives;
        }
        Vec::new()
    }

    /// Get security score for a package
    pub async fn get_security_score(&self, package_name: &str) -> Option<f32> {
        if let Some(security_data) = self.get_security_info().await {
            return security_data.security_scores
                .get(package_name)
                .and_then(|info| info.score);
        }
        None
    }

    /// Check if package has security vulnerabilities
    pub async fn has_security_issues(&self, package_name: &str) -> bool {
        if let Some(security_data) = self.get_security_info().await {
            if let Some(security_info) = security_data.security_scores.get(package_name) {
                return !security_info.vulnerabilities.is_empty() || security_info.cve_count > 0;
            }
        }
        false
    }

    /// Get packages by category
    pub async fn get_packages_by_category(&self, category: &str) -> Vec<PopularPackage> {
        if let Some(popular_data) = self.get_popular_packages().await {
            return popular_data.popular_packages
                .into_iter()
                .filter(|pkg| pkg.category == category)
                .collect();
        }
        Vec::new()
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        
        let mut popular_cache = self.popular_cache.write().await;
        *popular_cache = None;
        
        let mut mappings_cache = self.mappings_cache.write().await;
        *mappings_cache = None;
        
        let mut security_cache = self.security_cache.write().await;
        *security_cache = None;
        
        info!("Package discovery cache cleared");
    }
}

impl Default for PackageDiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}