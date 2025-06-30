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
            let mut config: OmniConfig = serde_yaml::from_str(&content)?;
            
            // Validate and update config if needed
            config.validate_and_fix();
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
    
    /// Validate configuration and fix any issues
    pub fn validate_and_fix(&mut self) {
        // Ensure max_parallel_jobs is reasonable
        if self.general.max_parallel_jobs == 0 {
            self.general.max_parallel_jobs = 1;
        } else if self.general.max_parallel_jobs > 16 {
            self.general.max_parallel_jobs = 16;
        }
        
        // Validate log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.general.log_level.as_str()) {
            self.general.log_level = "info".to_string();
        }
        
        // Ensure preferred_order has at least one entry
        if self.boxes.preferred_order.is_empty() {
            self.boxes.preferred_order = vec!["apt".to_string()];
        }
        
        // Validate GUI theme
        let valid_themes = ["dark", "light", "auto"];
        if !valid_themes.contains(&self.ui.gui_theme.as_str()) {
            self.ui.gui_theme = "dark".to_string();
        }
    }
    
    /// Update a specific configuration value
    pub fn set_general_option(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "auto_update" => self.general.auto_update = value.parse()?,
            "parallel_installs" => self.general.parallel_installs = value.parse()?,
            "max_parallel_jobs" => {
                let jobs: usize = value.parse()?;
                self.general.max_parallel_jobs = jobs.clamp(1, 16);
            },
            "confirm_installs" => self.general.confirm_installs = value.parse()?,
            "log_level" => {
                let valid_levels = ["error", "warn", "info", "debug", "trace"];
                if valid_levels.contains(&value) {
                    self.general.log_level = value.to_string();
                } else {
                    return Err(anyhow::anyhow!("Invalid log level: {}", value));
                }
            },
            "fallback_enabled" => self.general.fallback_enabled = value.parse()?,
            _ => return Err(anyhow::anyhow!("Unknown general option: {}", key)),
        }
        Ok(())
    }
    
    /// Add a package manager to preferred order
    pub fn add_preferred_box(&mut self, box_name: String) {
        if !self.boxes.preferred_order.contains(&box_name) {
            self.boxes.preferred_order.push(box_name);
        }
    }
    
    /// Remove a package manager from preferred order
    pub fn remove_preferred_box(&mut self, box_name: &str) {
        self.boxes.preferred_order.retain(|name| name != box_name);
    }
    
    /// Enable/disable a package manager
    pub fn set_box_enabled(&mut self, box_name: &str, enabled: bool) {
        if enabled {
            self.boxes.disabled_boxes.retain(|name| name != box_name);
        } else {
            let box_name = box_name.to_string();
            if !self.boxes.disabled_boxes.contains(&box_name) {
                self.boxes.disabled_boxes.push(box_name);
            }
        }
    }
    
    /// Get runtime directory for temporary files
    pub fn runtime_dir() -> Result<PathBuf> {
        let runtime_dir = dirs::runtime_dir()
            .or_else(|| dirs::cache_dir())
            .ok_or_else(|| anyhow::anyhow!("Could not find runtime directory"))?;
        Ok(runtime_dir.join("omni"))
    }
    
    /// Get log directory
    pub fn log_dir() -> Result<PathBuf> {
        let log_dir = Self::data_dir()?.join("logs");
        fs::create_dir_all(&log_dir)?;
        Ok(log_dir)
    }
    
    /// Merge with another config (for updates)
    pub fn merge(&mut self, other: &OmniConfig) {
        // Only merge non-default values to preserve user settings
        let default_config = Self::default();
        
        if other.general.auto_update != default_config.general.auto_update {
            self.general.auto_update = other.general.auto_update;
        }
        if other.general.parallel_installs != default_config.general.parallel_installs {
            self.general.parallel_installs = other.general.parallel_installs;
        }
        if other.general.max_parallel_jobs != default_config.general.max_parallel_jobs {
            self.general.max_parallel_jobs = other.general.max_parallel_jobs;
        }
        if other.general.confirm_installs != default_config.general.confirm_installs {
            self.general.confirm_installs = other.general.confirm_installs;
        }
        if other.general.log_level != default_config.general.log_level {
            self.general.log_level = other.general.log_level.clone();
        }
        if other.general.fallback_enabled != default_config.general.fallback_enabled {
            self.general.fallback_enabled = other.general.fallback_enabled;
        }
    }
    
    /// Create backup of current config
    pub fn backup(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let backup_path = config_path.with_extension("yaml.backup");
        
        if config_path.exists() {
            fs::copy(&config_path, &backup_path)?;
        }
        
        Ok(())
    }
    
    /// Restore from backup
    pub fn restore_backup() -> Result<()> {
        let config_path = Self::config_path()?;
        let backup_path = config_path.with_extension("yaml.backup");
        
        if backup_path.exists() {
            fs::copy(&backup_path, &config_path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No backup file found"))
        }
    }
}