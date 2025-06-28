use crate::secure_executor::{SecureExecutor, ExecutionConfig};
use crate::transaction::{TransactionManager, TransactionType, TransactionResult};
use crate::advanced_resolver::{AdvancedDependencyResolver, ResolutionStrategy};
use crate::error_handling::{OmniError, RecoveryManager};
use crate::input_validation::InputValidator;
use crate::privilege_manager::PrivilegeManager;
use crate::security::{SecurityVerifier, SecurityPolicy, VerificationResult};
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::snapshot::SnapshotManager;
use crate::manifest::OmniManifest;
use crate::boxes::apt::AptManager;

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::Utc;
use semver::VersionReq;

/// Secure and improved OmniBrain with proper error handling and security
pub struct SecureOmniBrain {
    mock_mode: bool,
    db: Option<Database>,
    snapshot_manager: Option<SnapshotManager>,
    transaction_manager: Option<TransactionManager>,
    dependency_resolver: Option<AdvancedDependencyResolver>,
    security_verifier: SecurityVerifier,
    privilege_manager: PrivilegeManager,
    recovery_manager: RecoveryManager,
    secure_executor: SecureExecutor,
}

impl SecureOmniBrain {
    pub fn new() -> Result<Self> {
        let mut privilege_manager = PrivilegeManager::new();
        privilege_manager.store_credentials();
        
        let security_policy = SecurityPolicy::default();
        let security_verifier = SecurityVerifier::new(security_policy);
        let recovery_manager = RecoveryManager::new();
        let secure_executor = SecureExecutor::new()?;
        
        Ok(SecureOmniBrain {
            mock_mode: false,
            db: None,
            snapshot_manager: None,
            transaction_manager: None,
            dependency_resolver: None,
            security_verifier,
            privilege_manager,
            recovery_manager,
            secure_executor,
        })
    }
    
    pub fn new_with_mock(mock_mode: bool) -> Result<Self> {
        let mut brain = Self::new()?;
        brain.mock_mode = mock_mode;
        Ok(brain)
    }
    
    async fn ensure_initialized(&mut self) -> Result<()> {
        if self.db.is_none() {
            self.db = Some(Database::new().await?);
        }
        if self.snapshot_manager.is_none() {
            self.snapshot_manager = Some(SnapshotManager::new().await?);
        }
        if self.transaction_manager.is_none() {
            self.transaction_manager = Some(TransactionManager::new().await?);
        }
        if self.dependency_resolver.is_none() {
            self.dependency_resolver = Some(
                AdvancedDependencyResolver::new(ResolutionStrategy::Latest).await?
            );
        }
        Ok(())
    }
    
    /// Secure package installation with full validation and transaction support
    pub async fn secure_install(
        &mut self,
        packages: &[String],
        box_type: Option<&str>,
        constraints: Option<HashMap<String, String>>,
    ) -> Result<TransactionResult> {
        info!("Starting secure installation of {} packages", packages.len());
        
        // Validate all inputs
        for package in packages {
            InputValidator::validate_package_name(package)?;
        }
        
        if let Some(bt) = box_type {
            InputValidator::validate_box_type(bt)?;
        }
        
        if self.mock_mode {
            return self.mock_install(packages).await;
        }
        
        self.ensure_initialized().await?;
        
        // Start transaction
        let transaction_id = self.transaction_manager
            .as_mut()
            .unwrap()
            .begin_transaction(
                TransactionType::Install,
                format!("Install {} packages", packages.len()),
            )
            .await?;
        
        let mut transaction_result = None;
        
        // Use recovery manager for resilient execution
        let result = self.recovery_manager.execute_with_recovery(|| {
            let packages = packages.to_vec();
            let box_type = box_type.map(|s| s.to_string());
            let constraints = constraints.clone();
            
            async move {
                self.execute_secure_install_transaction(packages, box_type, constraints).await
            }
        }).await;
        
        // Handle transaction result
        match result {
            Ok(tx_result) => {
                transaction_result = Some(tx_result);
            }
            Err(e) => {
                error!("Installation transaction failed: {}", e);
                
                // Attempt recovery
                if let Err(recovery_error) = self.recovery_manager.attempt_recovery(&e.downcast_ref::<OmniError>().unwrap_or(&OmniError::Unknown {
                    message: e.to_string()
                })).await {
                    error!("Recovery failed: {}", recovery_error);
                }
                
                // Rollback transaction
                if let Some(tm) = &mut self.transaction_manager {
                    let _ = tm.rollback_transaction().await;
                }
                
                return Err(e);
            }
        }
        
        Ok(transaction_result.unwrap())
    }
    
    async fn execute_secure_install_transaction(
        &mut self,
        packages: Vec<String>,
        box_type: Option<String>,
        constraints: Option<HashMap<String, String>>,
    ) -> Result<TransactionResult> {
        // Resolve dependencies
        let version_constraints = self.parse_version_constraints(constraints)?;
        
        let resolution_plan = self.dependency_resolver
            .as_mut()
            .unwrap()
            .resolve_dependencies(&packages, version_constraints)
            .await?;
        
        info!("Resolved {} packages for installation", resolution_plan.packages.len());
        
        // Check for conflicts
        if !resolution_plan.conflicts.is_empty() {
            warn!("Found {} conflicts in resolution plan", resolution_plan.conflicts.len());
            for conflict in &resolution_plan.conflicts {
                warn!("Conflict: {}", conflict.description);
            }
            
            // For now, fail if there are conflicts
            // In a full implementation, this could prompt for user resolution
            return Err(OmniError::InstallationFailed {
                package: "multiple".to_string(),
                box_type: "unknown".to_string(),
                reason: format!("Dependency conflicts: {}", 
                    resolution_plan.conflicts.iter()
                        .map(|c| &c.description)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }.into());
        }
        
        // Add operations to transaction
        let tm = self.transaction_manager.as_mut().unwrap();
        for package in &resolution_plan.packages {
            tm.add_operation(
                TransactionType::Install,
                package.name.clone(),
                package.box_type.clone(),
                None, // No previous version for new installs
            ).await?;
        }
        
        // Execute transaction
        let transaction_result = tm.commit_transaction().await?;
        
        // Record results in database
        if let Some(db) = &self.db {
            for package in &resolution_plan.packages {
                let install_record = InstallRecord {
                    id: Uuid::new_v4().to_string(),
                    package_name: package.name.clone(),
                    box_type: package.box_type.clone(),
                    version: Some(package.version.to_string()),
                    source_url: package.source_url.clone(),
                    install_path: None,
                    installed_at: Utc::now(),
                    status: if transaction_result.is_successful() {
                        InstallStatus::Success
                    } else {
                        InstallStatus::Failed
                    },
                    metadata: Some(format!("Transaction: {}", transaction_result.transaction_id)),
                };
                
                let _ = db.record_install(&install_record).await;
            }
        }
        
        if transaction_result.is_successful() {
            info!("✅ Secure installation completed successfully");
        } else if transaction_result.is_rolled_back() {
            warn!("⚠️ Installation was rolled back due to errors");
        } else {
            error!("❌ Installation failed");
        }
        
        Ok(transaction_result)
    }
    
    async fn mock_install(&self, packages: &[String]) -> Result<TransactionResult> {
        println!("🎭 [MOCK] Starting secure installation of {} packages", packages.len());
        
        for package in packages {
            println!("🎭 [MOCK] Installing '{}'", package);
            println!("✅ [MOCK] Successfully installed {} (simulated)", package);
        }
        
        Ok(TransactionResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: crate::transaction::TransactionStatus::Completed,
            successful_operations: packages.len(),
            failed_operations: 0,
            errors: vec![],
        })
    }
    
    /// Secure package removal with transaction support
    pub async fn secure_remove(
        &mut self,
        packages: &[String],
        box_type: Option<&str>,
    ) -> Result<TransactionResult> {
        info!("Starting secure removal of {} packages", packages.len());
        
        // Validate inputs
        for package in packages {
            InputValidator::validate_package_name(package)?;
        }
        
        if let Some(bt) = box_type {
            InputValidator::validate_box_type(bt)?;
        }
        
        if self.mock_mode {
            return self.mock_remove(packages).await;
        }
        
        self.ensure_initialized().await?;
        
        // Start transaction
        let transaction_id = self.transaction_manager
            .as_mut()
            .unwrap()
            .begin_transaction(
                TransactionType::Remove,
                format!("Remove {} packages", packages.len()),
            )
            .await?;
        
        // Add operations to transaction
        let tm = self.transaction_manager.as_mut().unwrap();
        for package in packages {
            // Get current version if installed
            let current_version = if let Some(db) = &self.db {
                db.get_installed_packages().await?
                    .iter()
                    .find(|r| &r.package_name == package)
                    .map(|r| r.version.clone().unwrap_or_default())
            } else {
                None
            };
            
            tm.add_operation(
                TransactionType::Remove,
                package.clone(),
                box_type.unwrap_or("auto").to_string(),
                current_version,
            ).await?;
        }
        
        // Execute transaction
        let transaction_result = tm.commit_transaction().await?;
        
        if transaction_result.is_successful() {
            info!("✅ Secure removal completed successfully");
        } else {
            error!("❌ Removal failed or was rolled back");
        }
        
        Ok(transaction_result)
    }
    
    async fn mock_remove(&self, packages: &[String]) -> Result<TransactionResult> {
        println!("🎭 [MOCK] Starting secure removal of {} packages", packages.len());
        
        for package in packages {
            println!("🎭 [MOCK] Removing '{}'", package);
            println!("✅ [MOCK] Successfully removed {} (simulated)", package);
        }
        
        Ok(TransactionResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: crate::transaction::TransactionStatus::Completed,
            successful_operations: packages.len(),
            failed_operations: 0,
            errors: vec![],
        })
    }
    
    /// Install from manifest with enhanced security and dependency resolution
    pub async fn secure_install_from_manifest(
        &mut self,
        manifest: OmniManifest,
    ) -> Result<TransactionResult> {
        info!("Starting secure manifest installation: {}", manifest.project);
        
        if self.mock_mode {
            return self.mock_manifest_install(&manifest).await;
        }
        
        self.ensure_initialized().await?;
        
        // Extract packages from manifest
        let packages: Vec<String> = manifest.apps.iter()
            .map(|app| app.name.clone())
            .collect();
        
        // Create constraints from manifest
        let mut constraints = HashMap::new();
        for app in &manifest.apps {
            if let Some(ref version) = app.version {
                constraints.insert(app.name.clone(), version.clone());
            }
        }
        
        // Use secure install with manifest packages
        self.secure_install(&packages, None, Some(constraints)).await
    }
    
    async fn mock_manifest_install(&self, manifest: &OmniManifest) -> Result<TransactionResult> {
        println!("🎭 [MOCK] Installing from manifest: {}", manifest.project);
        if let Some(desc) = &manifest.description {
            println!("📋 [MOCK] Description: {}", desc);
        }
        
        for app in &manifest.apps {
            println!("🎭 [MOCK] Installing {} via {} box", app.name, app.box_type);
            if let Some(source) = &app.source {
                println!("📦 [MOCK] Source: {}", source);
            }
            println!("✅ [MOCK] Successfully installed {} (simulated)", app.name);
        }
        
        Ok(TransactionResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: crate::transaction::TransactionStatus::Completed,
            successful_operations: manifest.apps.len(),
            failed_operations: 0,
            errors: vec![],
        })
    }
    
    /// Search packages with security filtering
    pub async fn secure_search(
        &self,
        query: &str,
        box_type: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<crate::search::SearchResult>> {
        // Validate search query
        if query.is_empty() || query.len() > 255 {
            return Err(OmniError::ValidationError {
                field: "query".to_string(),
                message: "Search query must be 1-255 characters".to_string(),
            }.into());
        }
        
        // Basic sanitization - remove dangerous characters
        let sanitized_query = query.chars()
            .filter(|c| c.is_alphanumeric() || " -._+".contains(*c))
            .collect::<String>();
        
        if sanitized_query.is_empty() {
            return Err(OmniError::ValidationError {
                field: "query".to_string(),
                message: "Query contains no valid characters".to_string(),
            }.into());
        }
        
        info!("Performing secure search for: '{}'", sanitized_query);
        
        // For now, return empty results - in real implementation would use SearchEngine
        // with proper security filtering
        Ok(vec![])
    }
    
    /// Get secure system status
    pub async fn get_system_status(&mut self) -> Result<SystemStatus> {
        self.ensure_initialized().await?;
        
        let mut status = SystemStatus {
            total_packages: 0,
            failed_packages: 0,
            pending_updates: 0,
            last_update: None,
            security_warnings: vec![],
            storage_usage: None,
        };
        
        // Get package statistics
        if let Some(db) = &self.db {
            let installed = db.get_installed_packages().await?;
            status.total_packages = installed.len();
            status.failed_packages = installed.iter()
                .filter(|p| p.status == InstallStatus::Failed)
                .count();
        }
        
        // Check for security issues
        status.security_warnings = self.check_security_warnings().await?;
        
        Ok(status)
    }
    
    async fn check_security_warnings(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        // Check if running as root unnecessarily
        if PrivilegeManager::is_root() {
            warnings.push("Running as root user - consider using privilege dropping".to_string());
        }
        
        // Check for insecure configurations
        // This would check various security settings
        
        Ok(warnings)
    }
    
    fn parse_version_constraints(
        &self,
        constraints: Option<HashMap<String, String>>,
    ) -> Result<HashMap<String, VersionReq>> {
        let mut parsed = HashMap::new();
        
        if let Some(constraints) = constraints {
            for (package, version_str) in constraints {
                match VersionReq::parse(&version_str) {
                    Ok(req) => {
                        parsed.insert(package, req);
                    }
                    Err(e) => {
                        warn!("Invalid version requirement '{}' for package '{}': {}", 
                              version_str, package, e);
                        // Use any version as fallback
                        parsed.insert(package, VersionReq::parse("*").unwrap());
                    }
                }
            }
        }
        
        Ok(parsed)
    }
}

/// System status information
#[derive(Debug)]
pub struct SystemStatus {
    pub total_packages: usize,
    pub failed_packages: usize,
    pub pending_updates: usize,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
    pub security_warnings: Vec<String>,
    pub storage_usage: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_secure_brain_creation() {
        let brain = SecureOmniBrain::new();
        assert!(brain.is_ok());
    }
    
    #[tokio::test]
    async fn test_mock_install() {
        let mut brain = SecureOmniBrain::new_with_mock(true).unwrap();
        let packages = vec!["firefox".to_string(), "vim".to_string()];
        
        let result = brain.secure_install(&packages, Some("apt"), None).await;
        assert!(result.is_ok());
        
        let tx_result = result.unwrap();
        assert!(tx_result.is_successful());
        assert_eq!(tx_result.successful_operations, 2);
    }
    
    #[tokio::test]
    async fn test_input_validation() {
        let mut brain = SecureOmniBrain::new().unwrap();
        
        // Invalid package name
        let result = brain.secure_install(&vec!["../../../etc/passwd".to_string()], None, None).await;
        assert!(result.is_err());
        
        // Invalid box type
        let result = brain.secure_install(&vec!["firefox".to_string()], Some("malicious"), None).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_search_sanitization() {
        let brain = SecureOmniBrain::new().unwrap();
        
        // Valid search
        let result = brain.secure_search("firefox web browser", None, None).await;
        assert!(result.is_ok());
        
        // Invalid search with dangerous characters
        let result = brain.secure_search("test; rm -rf /", None, None).await;
        assert!(result.is_err());
        
        // Empty search
        let result = brain.secure_search("", None, None).await;
        assert!(result.is_err());
    }
}