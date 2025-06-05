
use serde::Deserialize;
use std::fs;

/// Represents the overall structure of an Omni manifest file (typically YAML).
/// A manifest describes a project and the applications (apps) it requires.
#[derive(Debug, Deserialize)]
pub struct OmniManifest {
    /// The name of the project defined by this manifest.
    pub project: String,
    /// An optional description of the project.
    pub description: Option<String>,
    /// A list of applications to be managed by Omni for this project.
    pub apps: Vec<OmniApp>,
    /// Optional metadata about the manifest file itself.
    pub meta: Option<MetaInfo>,
}

/// Defines a single application entry within the Omni manifest.
#[derive(Debug, Deserialize)]
pub struct OmniApp {
    /// The primary name of the application. This is often the package name
    /// used by the respective package manager.
    pub name: String,
    /// The type of "box" (package manager or installation method) to use for this app.
    /// Examples: "apt", "pacman", "dnf", "flatpak", "appimage".
    /// In the manifest file (e.g., YAML), this field is expected to be named "box".
    #[serde(rename = "box")]
    pub box_type: String,
    /// An optional version string for the application.
    /// (Currently, Omni's core logic does not strictly enforce versioning during install).
    pub version: Option<String>,
    /// An optional source string, which can have different meanings based on the box_type.
    /// For Flatpaks, this might be the Flathub ID if different from `name`.
    /// For AppImages, this could be a direct download URL or a path.
    pub source: Option<String>,
}

/// Contains metadata about the manifest file.
#[derive(Debug, Deserialize)]
pub struct MetaInfo {
    /// Optional field for who created the manifest.
    pub created_by: Option<String>,
    /// Optional field for when the manifest was created.
    pub created_on: Option<String>,
    /// If `true`, Omni will attempt to install applications using the system's native
    /// package manager if the specified `box_type` is unavailable or fails.
    /// Defaults to `false` if not present.
    pub distro_fallback: Option<bool>,
}

impl OmniManifest {
    /// Loads an `OmniManifest` from a YAML file at the given path.
    ///
    /// # Arguments
    /// * `path` - The filesystem path to the manifest file.
    ///
    /// # Returns
    /// * `Ok(Self)` if the file is successfully read and parsed as YAML.
    /// * `Err(Box<dyn std::error::Error>)` if file reading or YAML parsing fails.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the entire file content into a string.
        let content = fs::read_to_string(path)?;
        // Deserialize the YAML string into an OmniManifest struct.
        let manifest: OmniManifest = serde_yaml::from_str(&content)?;
        Ok(manifest)
    }
}
