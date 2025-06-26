use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use dirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OmniConfig {
    pub general: GeneralConfig,
    pub boxes: BoxConfig,
    pub security: SecurityConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub auto_update: bool,
    pub parallel_installs: bool,
    pub max_parallel_jobs: usize,
    pub confirm_installs: bool,
    pub log_level: String,
    pub fallback_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoxConfig {
    pub preferred_order: Vec<String>,
    pub disabled_boxes: Vec<String>,
    pub apt_options: Vec<String>,
    pub dnf_options: Vec<String>,
    pub pacman_options: Vec<String>,
    pub snap_options: Vec<String>,
    pub flatpak_options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub verify_signatures: bool,
    pub verify_checksums: bool,
    pub allow_untrusted: bool,
    pub check_mirrors: bool,
    pub signature_servers: Vec<String>,
    pub trusted_keys: Vec<String>,
    pub interactive_prompts: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub show_progress: bool,
    pub use_colors: bool,
    pub compact_output: bool,
    pub gui_theme: String,
}

impl Default for OmniConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                auto_update: false,
                parallel_installs: true,
                max_parallel_jobs: 4,
                confirm_installs: true,
                log_level: "info".to_string(),
                fallback_enabled: true,
            },
            boxes: BoxConfig {
                preferred_order: vec![
                    "apt".to_string(),
                    "dnf".to_string(),
                    "pacman".to_string(),
                    "flatpak".to_string(),
                    "snap".to_string(),
                    "appimage".to_string(),
                ],
                disabled_boxes: vec![],
                apt_options: vec!["-y".to_string()],
                dnf_options: vec!["-y".to_string()],
                pacman_options: vec!["--noconfirm".to_string()],
                snap_options: vec![],
                flatpak_options: vec!["-y".to_string()],
            },
            security: SecurityConfig {
                verify_signatures: true,
                verify_checksums: true,
                allow_untrusted: false,
                check_mirrors: true,
                signature_servers: vec![
                    "keyserver.ubuntu.com".to_string(),
                    "keys.openpgp.org".to_string(),
                    "pgp.mit.edu".to_string(),
                ],
                trusted_keys: vec![],
                interactive_prompts: true,
            },
            ui: UiConfig {
                show_progress: true,
                use_colors: true,
                compact_output: false,
                gui_theme: "dark".to_string(),
            },
        }
    }
}

impl OmniConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: OmniConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("omni").join("config.yaml"))
    }
    
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        Ok(data_dir.join("omni"))
    }
    
    pub fn cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?;
        Ok(cache_dir.join("omni"))
    }
    
    pub fn is_box_enabled(&self, box_name: &str) -> bool {
        !self.boxes.disabled_boxes.contains(&box_name.to_string())
    }
    
    pub fn get_box_priority(&self, box_name: &str) -> Option<usize> {
        self.boxes.preferred_order
            .iter()
            .position(|name| name == box_name)
    }
}