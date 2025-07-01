use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct OmniManifest {
    #[allow(dead_code)]
    pub project: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub apps: Vec<OmniApp>,
    pub meta: Option<MetaInfo>,
}

#[derive(Debug, Deserialize)]
pub struct OmniApp {
    pub name: String,
    #[serde(rename = "box")]
    pub box_type: String,
    #[allow(dead_code)]
    pub version: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaInfo {
    #[allow(dead_code)]
    pub created_by: Option<String>,
    #[allow(dead_code)]
    pub created_on: Option<String>,
    pub distro_fallback: Option<bool>,
}

impl OmniManifest {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let manifest: OmniManifest = serde_yaml::from_str(&content)?;
        Ok(manifest)
    }
}
