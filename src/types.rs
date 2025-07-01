/// Common types used across the Omni package management system
///
/// This module provides unified type definitions that are shared across
/// different package managers to ensure consistency and avoid duplication.
use serde::{Deserialize, Serialize};

/// Represents an installed package with common metadata
///
/// This struct is used across all package managers to provide a consistent
/// representation of installed packages, regardless of the underlying
/// package management system (APT, DNF, Snap, Brew, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// The name of the package
    pub name: String,

    /// The version of the installed package
    pub version: String,

    /// Optional package description
    pub description: Option<String>,

    /// Optional package size in bytes
    pub size: Option<u64>,

    /// Optional architecture (e.g., "amd64", "arm64", "noarch")
    pub architecture: Option<String>,

    /// Optional installation source/repository
    pub source: Option<String>,

    /// Optional installation date as Unix timestamp
    pub install_date: Option<u64>,

    /// Optional package category or group
    pub category: Option<String>,

    /// Optional maintainer or vendor information
    pub maintainer: Option<String>,

    /// Optional homepage URL
    pub homepage: Option<String>,

    /// Whether this package was explicitly installed by the user
    /// (as opposed to auto-installed as a dependency)
    pub explicit: Option<bool>,
}

impl InstalledPackage {
    /// Create a new InstalledPackage with minimal required fields
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: None,
            size: None,
            architecture: None,
            source: None,
            install_date: None,
            category: None,
            maintainer: None,
            homepage: None,
            explicit: None,
        }
    }

    /// Create a new InstalledPackage with name, version, and description
    pub fn with_description(name: String, version: String, description: Option<String>) -> Self {
        Self {
            name,
            version,
            description,
            size: None,
            architecture: None,
            source: None,
            install_date: None,
            category: None,
            maintainer: None,
            homepage: None,
            explicit: None,
        }
    }

    /// Set the package size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the package architecture
    pub fn with_architecture(mut self, architecture: String) -> Self {
        self.architecture = Some(architecture);
        self
    }

    /// Set the package source/repository
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Set the installation date
    pub fn with_install_date(mut self, timestamp: u64) -> Self {
        self.install_date = Some(timestamp);
        self
    }

    /// Set the package category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Set the maintainer information
    pub fn with_maintainer(mut self, maintainer: String) -> Self {
        self.maintainer = Some(maintainer);
        self
    }

    /// Set the homepage URL
    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    /// Mark whether this package was explicitly installed
    pub fn with_explicit(mut self, explicit: bool) -> Self {
        self.explicit = Some(explicit);
        self
    }

    /// Get a display-friendly string representation
    pub fn display(&self) -> String {
        match &self.description {
            Some(desc) => format!("{} {} - {}", self.name, self.version, desc),
            None => format!("{} {}", self.name, self.version),
        }
    }

    /// Check if this package matches a given name (case-insensitive)
    pub fn matches_name(&self, name: &str) -> bool {
        self.name.to_lowercase() == name.to_lowercase()
    }

    /// Get the package identifier as "name-version"
    pub fn identifier(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

impl std::fmt::Display for InstalledPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installed_package_creation() {
        let pkg = InstalledPackage::new("curl".to_string(), "7.68.0".to_string());
        assert_eq!(pkg.name, "curl");
        assert_eq!(pkg.version, "7.68.0");
        assert_eq!(pkg.description, None);
    }

    #[test]
    fn test_installed_package_with_description() {
        let pkg = InstalledPackage::with_description(
            "curl".to_string(),
            "7.68.0".to_string(),
            Some("Command line tool for transferring data".to_string()),
        );
        assert_eq!(pkg.name, "curl");
        assert_eq!(pkg.version, "7.68.0");
        assert!(pkg.description.is_some());
    }

    #[test]
    fn test_installed_package_builder() {
        let pkg = InstalledPackage::new("nginx".to_string(), "1.18.0".to_string())
            .with_size(1024)
            .with_architecture("amd64".to_string())
            .with_explicit(true);

        assert_eq!(pkg.size, Some(1024));
        assert_eq!(pkg.architecture, Some("amd64".to_string()));
        assert_eq!(pkg.explicit, Some(true));
    }

    #[test]
    fn test_matches_name() {
        let pkg = InstalledPackage::new("Curl".to_string(), "7.68.0".to_string());
        assert!(pkg.matches_name("curl"));
        assert!(pkg.matches_name("CURL"));
        assert!(pkg.matches_name("Curl"));
        assert!(!pkg.matches_name("wget"));
    }

    #[test]
    fn test_identifier() {
        let pkg = InstalledPackage::new("curl".to_string(), "7.68.0".to_string());
        assert_eq!(pkg.identifier(), "curl-7.68.0");
    }

    #[test]
    fn test_display() {
        let pkg = InstalledPackage::with_description(
            "curl".to_string(),
            "7.68.0".to_string(),
            Some("HTTP client".to_string()),
        );
        assert_eq!(pkg.display(), "curl 7.68.0 - HTTP client");

        let pkg_no_desc = InstalledPackage::new("curl".to_string(), "7.68.0".to_string());
        assert_eq!(pkg_no_desc.display(), "curl 7.68.0");
    }

    #[test]
    fn test_serialization() {
        let pkg = InstalledPackage::new("curl".to_string(), "7.68.0".to_string())
            .with_description(Some("HTTP client".to_string()));

        // Test JSON serialization
        let json = serde_json::to_string(&pkg).unwrap();
        let deserialized: InstalledPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(pkg, deserialized);
    }
}
