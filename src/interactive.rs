use anyhow::Result;
use dialoguer::{
    Confirm, Input, Select, MultiSelect, FuzzySelect,
    theme::ColorfulTheme,
};
use crate::resolver::{ResolutionPlan, ResolvedPackage};
use crate::security::{VerificationResult, TrustLevel};
use crate::search::SearchResult;
use tracing::{info, warn};
use std::fmt::Display;

pub struct InteractivePrompts {
    theme: ColorfulTheme,
}

#[derive(Debug, Clone)]
pub struct InstallConfirmation {
    pub proceed: bool,
    pub selected_packages: Vec<String>,
    pub skip_verification: bool,
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub action: ConflictAction,
    pub selected_option: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConflictAction {
    Abort,
    Force,
    SelectAlternative,
    Skip,
}

impl InteractivePrompts {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }
    
    pub fn confirm_installation(&self, plan: &ResolutionPlan) -> Result<InstallConfirmation> {
        if plan.packages.is_empty() {
            return Ok(InstallConfirmation {
                proceed: false,
                selected_packages: vec![],
                skip_verification: false,
            });
        }
        
        println!("\n📦 Installation Plan:");
        println!("{}", "─".repeat(50));
        
        // Show packages to be installed
        for (i, package) in plan.packages.iter().enumerate() {
            let status = if i == 0 { "📍 Target" } else { "🔗 Dependency" };
            println!("{} {} [{}] v{}", 
                status, 
                package.name, 
                package.box_type, 
                package.version
            );
        }
        
        // Show total size if available
        if let Some(total_size) = plan.total_size {
            println!("\n💾 Total download size: {}", format_size(total_size));
        }
        
        // Show conflicts if any
        if !plan.conflicts.is_empty() {
            println!("\n⚠️  Conflicts detected:");
            for conflict in &plan.conflicts {
                println!("   • {}", conflict);
            }
        }
        
        // Show warnings if any
        if !plan.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &plan.warnings {
                println!("   • {}", warning);
            }
        }
        
        println!();
        
        // Ask for confirmation
        let proceed = if !plan.conflicts.is_empty() {
            let options = vec![
                "Abort installation",
                "Force installation (ignore conflicts)",
                "Show alternatives",
            ];
            
            let selection = Select::with_theme(&self.theme)
                .with_prompt("Conflicts detected. What would you like to do?")
                .items(&options)
                .default(0)
                .interact()?;
            
            match selection {
                0 => false,
                1 => {
                    Confirm::with_theme(&self.theme)
                        .with_prompt("⚠️  Are you sure you want to force installation despite conflicts?")
                        .default(false)
                        .interact()?
                }
                2 => {
                    // This would trigger alternative search
                    return Ok(InstallConfirmation {
                        proceed: false,
                        selected_packages: vec!["__show_alternatives__".to_string()],
                        skip_verification: false,
                    });
                }
                _ => false,
            }
        } else {
            Confirm::with_theme(&self.theme)
                .with_prompt("Do you want to proceed with the installation?")
                .default(true)
                .interact()?
        };
        
        if !proceed {
            return Ok(InstallConfirmation {
                proceed: false,
                selected_packages: vec![],
                skip_verification: false,
            });
        }
        
        // Ask about optional dependencies
        let optional_deps: Vec<&ResolvedPackage> = plan.packages
            .iter()
            .filter(|p| p.dependencies.iter().any(|d| d.optional))
            .collect();
        
        let mut selected_packages: Vec<String> = plan.packages
            .iter()
            .filter(|p| !p.dependencies.iter().any(|d| d.optional))
            .map(|p| p.name.clone())
            .collect();
        
        if !optional_deps.is_empty() {
            println!("\n🔧 Optional dependencies found:");
            let optional_names: Vec<String> = optional_deps
                .iter()
                .map(|p| format!("{} [{}] - {}", p.name, p.box_type, "Optional"))
                .collect();
            
            let selected_indices = MultiSelect::with_theme(&self.theme)
                .with_prompt("Select optional dependencies to install")
                .items(&optional_names)
                .interact()?;
            
            for index in selected_indices {
                selected_packages.push(optional_deps[index].name.clone());
            }
        }
        
        Ok(InstallConfirmation {
            proceed: true,
            selected_packages,
            skip_verification: false,
        })
    }
    
    pub fn confirm_security_risk(&self, verification: &VerificationResult) -> Result<bool> {
        match verification.trust_level {
            TrustLevel::Trusted | TrustLevel::Valid => Ok(true),
            TrustLevel::Unsigned => {
                println!("\n🔒 Security Warning:");
                println!("{}", "─".repeat(50));
                println!("This package is not digitally signed.");
                println!("Details: {}", verification.details);
                
                if !verification.warnings.is_empty() {
                    println!("\nAdditional warnings:");
                    for warning in &verification.warnings {
                        println!("⚠️  {}", warning);
                    }
                }
                
                Confirm::with_theme(&self.theme)
                    .with_prompt("Do you want to continue with this unsigned package?")
                    .default(false)
                    .interact()
                    .map_err(|e| anyhow::anyhow!("Failed to get user confirmation: {}", e))
            }
            TrustLevel::Untrusted => {
                println!("\n🚨 Security Alert:");
                println!("{}", "─".repeat(50));
                println!("This package failed security verification!");
                println!("Details: {}", verification.details);
                
                for warning in &verification.warnings {
                    println!("❌ {}", warning);
                }
                
                let options = vec![
                    "Abort installation (recommended)",
                    "Continue anyway (dangerous)",
                ];
                
                let selection = Select::with_theme(&self.theme)
                    .with_prompt("Package verification failed. What would you like to do?")
                    .items(&options)
                    .default(0)
                    .interact()?;
                
                if selection == 1 {
                    Confirm::with_theme(&self.theme)
                        .with_prompt("⚠️  Are you absolutely sure you want to install this untrusted package?")
                        .default(false)
                        .interact()
                        .map_err(|e| anyhow::anyhow!("Failed to get user confirmation: {}", e))
                } else {
                    Ok(false)
                }
            }
        }
    }
    
    pub fn select_from_search_results(&self, results: &[SearchResult], query: &str) -> Result<Option<SearchResult>> {
        if results.is_empty() {
            println!("No packages found for query: '{}'", query);
            return Ok(None);
        }
        
        println!("\n🔍 Search Results for '{}':", query);
        println!("{}", "─".repeat(50));
        
        let items: Vec<String> = results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let status = if result.installed { "✅" } else { "  " };
                let desc = result.description
                    .as_ref()
                    .map(|d| {
                        if d.len() > 60 {
                            format!(" - {}...", &d[..57])
                        } else {
                            format!(" - {}", d)
                        }
                    })
                    .unwrap_or_default();
                
                format!("{} {}. {} [{}]{}", status, i + 1, result.name, result.box_type, desc)
            })
            .collect();
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a package to install (or ESC to cancel)")
            .items(&items)
            .interact_opt()?;
        
        if let Some(index) = selection {
            Ok(Some(results[index].clone()))
        } else {
            Ok(None)
        }
    }
    
    pub fn select_package_manager(&self, available: &[String]) -> Result<Option<String>> {
        if available.is_empty() {
            return Ok(None);
        }
        
        if available.len() == 1 {
            return Ok(Some(available[0].clone()));
        }
        
        println!("\n📦 Multiple package managers available:");
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select package manager")
            .items(available)
            .interact_opt()?;
        
        if let Some(index) = selection {
            Ok(Some(available[index].clone()))
        } else {
            Ok(None)
        }
    }
    
    pub fn resolve_conflict(&self, conflict: &str, alternatives: &[String]) -> Result<ConflictResolution> {
        println!("\n⚠️  Conflict Detected:");
        println!("{}", "─".repeat(50));
        println!("{}", conflict);
        
        if alternatives.is_empty() {
            let options = vec![
                "Abort installation",
                "Force installation (ignore conflict)",
                "Skip this package",
            ];
            
            let selection = Select::with_theme(&self.theme)
                .with_prompt("How would you like to resolve this conflict?")
                .items(&options)
                .default(0)
                .interact()?;
            
            let action = match selection {
                0 => ConflictAction::Abort,
                1 => ConflictAction::Force,
                2 => ConflictAction::Skip,
                _ => ConflictAction::Abort,
            };
            
            Ok(ConflictResolution {
                action,
                selected_option: None,
            })
        } else {
            println!("\nAvailable alternatives:");
            let mut options = alternatives.clone();
            options.extend(vec![
                "Abort installation".to_string(),
                "Force original (ignore conflict)".to_string(),
                "Skip this package".to_string(),
            ]);
            
            let selection = Select::with_theme(&self.theme)
                .with_prompt("Select an alternative or action")
                .items(&options)
                .default(0)
                .interact()?;
            
            if selection < alternatives.len() {
                Ok(ConflictResolution {
                    action: ConflictAction::SelectAlternative,
                    selected_option: Some(alternatives[selection].clone()),
                })
            } else {
                let action = match selection - alternatives.len() {
                    0 => ConflictAction::Abort,
                    1 => ConflictAction::Force,
                    2 => ConflictAction::Skip,
                    _ => ConflictAction::Abort,
                };
                
                Ok(ConflictResolution {
                    action,
                    selected_option: None,
                })
            }
        }
    }
    
    pub fn get_input<T>(&self, prompt: &str, default: Option<T>) -> Result<T>
    where
        T: Clone + Display + std::str::FromStr,
        T::Err: Display,
    {
        let mut input_builder = Input::with_theme(&self.theme)
            .with_prompt(prompt);
        
        if let Some(default_val) = default {
            input_builder = input_builder.default(default_val);
        }
        
        input_builder
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Input error: {}", e))
    }
    
    pub fn get_confirmation(&self, prompt: &str, default: bool) -> Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation error: {}", e))
    }
    
    pub fn fuzzy_select_package(&self, packages: &[String], prompt: &str) -> Result<Option<String>> {
        if packages.is_empty() {
            return Ok(None);
        }
        
        let selection = FuzzySelect::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(packages)
            .interact_opt()?;
        
        if let Some(index) = selection {
            Ok(Some(packages[index].clone()))
        } else {
            Ok(None)
        }
    }
    
    pub fn show_progress_with_confirmation(&self, message: &str) -> Result<bool> {
        println!("\n{}", message);
        
        Confirm::with_theme(&self.theme)
            .with_prompt("Continue?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Progress confirmation error: {}", e))
    }
    
    pub fn display_error_with_options(&self, error: &anyhow::Error, recoverable: bool) -> Result<bool> {
        println!("\n❌ Error occurred:");
        println!("{}", "─".repeat(50));
        println!("{}", error);
        
        // Show error chain if available
        let mut source = error.source();
        while let Some(err) = source {
            println!("  Caused by: {}", err);
            source = err.source();
        }
        
        if recoverable {
            let options = vec![
                "Retry operation",
                "Skip and continue",
                "Abort",
            ];
            
            let selection = Select::with_theme(&self.theme)
                .with_prompt("How would you like to proceed?")
                .items(&options)
                .default(0)
                .interact()?;
            
            match selection {
                0 => Ok(true),  // Retry
                1 => Ok(false), // Skip
                _ => Err(anyhow::anyhow!("Operation aborted by user")),
            }
        } else {
            println!("\nThis error is not recoverable.");
            let _ = Confirm::with_theme(&self.theme)
                .with_prompt("Press Enter to exit")
                .default(true)
                .interact();
            
            Err(anyhow::anyhow!("Unrecoverable error"))
        }
    }
    
    pub fn select_snapshot(&self, snapshots: &[(String, String, String)]) -> Result<Option<String>> {
        if snapshots.is_empty() {
            println!("No snapshots available.");
            return Ok(None);
        }
        
        println!("\n📸 Available snapshots:");
        
        let items: Vec<String> = snapshots
            .iter()
            .map(|(id, name, date)| format!("{} ({})", name, date))
            .collect();
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a snapshot to revert to")
            .items(&items)
            .interact_opt()?;
        
        if let Some(index) = selection {
            Ok(Some(snapshots[index].0.clone()))
        } else {
            Ok(None)
        }
    }
}

impl Default for InteractivePrompts {
    fn default() -> Self {
        Self::new()
    }
}

fn format_size(size: u64) -> String {
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

#[derive(Debug, thiserror::Error)]
pub enum OmniError {
    #[error("Package not found: {package}")]
    PackageNotFound { package: String },
    
    #[error("Dependency resolution failed: {reason}")]
    DependencyResolutionFailed { reason: String },
    
    #[error("Security verification failed: {details}")]
    SecurityVerificationFailed { details: String },
    
    #[error("Installation failed for {package}: {reason}")]
    InstallationFailed { package: String, reason: String },
    
    #[error("Network error: {details}")]
    NetworkError { details: String },
    
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    #[error("Configuration error: {details}")]
    ConfigurationError { details: String },
    
    #[error("Database error: {details}")]
    DatabaseError { details: String },
    
    #[error("User cancelled operation")]
    UserCancelled,
    
    #[error("Unsupported operation: {operation} for {box_type}")]
    UnsupportedOperation { operation: String, box_type: String },
}

impl OmniError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            OmniError::NetworkError { .. } => true,
            OmniError::InstallationFailed { .. } => true,
            OmniError::DependencyResolutionFailed { .. } => true,
            OmniError::PackageNotFound { .. } => false,
            OmniError::SecurityVerificationFailed { .. } => false,
            OmniError::PermissionDenied { .. } => false,
            OmniError::ConfigurationError { .. } => false,
            OmniError::DatabaseError { .. } => false,
            OmniError::UserCancelled => false,
            OmniError::UnsupportedOperation { .. } => false,
        }
    }
    
    pub fn suggested_action(&self) -> &'static str {
        match self {
            OmniError::PackageNotFound { .. } => "Try searching for similar package names",
            OmniError::DependencyResolutionFailed { .. } => "Check for conflicting packages or try manual installation",
            OmniError::SecurityVerificationFailed { .. } => "Verify package source or use --allow-untrusted flag",
            OmniError::InstallationFailed { .. } => "Check system requirements and available space",
            OmniError::NetworkError { .. } => "Check internet connection and try again",
            OmniError::PermissionDenied { .. } => "Run with appropriate privileges (sudo)",
            OmniError::ConfigurationError { .. } => "Check configuration file syntax",
            OmniError::DatabaseError { .. } => "Try clearing cache or rebuilding database",
            OmniError::UserCancelled => "Operation was cancelled",
            OmniError::UnsupportedOperation { .. } => "Use a different package manager",
        }
    }
}