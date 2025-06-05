
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct OmniManifest {
    pub project: String,
    pub description: Option<String>,
    pub apps: Vec<OmniApp>,
    pub meta: Option<MetaInfo>,
}

#[derive(Debug, Deserialize)]
pub struct OmniApp {
    pub name: String,
    #[serde(rename = "box")]
    pub box_type: String,
    pub version: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaInfo {
    pub created_by: Option<String>,
    pub created_on: Option<String>,
    pub distro_fallback: Option<bool>,
}

impl OmniManifest {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let manifest: OmniManifest = serde_yaml::from_str(&content)?;
        Ok(manifest)
    }
}
