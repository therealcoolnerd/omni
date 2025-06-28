use crate::error_handling::{OmniError, RecoveryManager};
use crate::database::{Database, InstallRecord, InstallStatus};
use crate::snapshot::SnapshotManager;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Transaction types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Install,
    Remove,
    Update,
    ManifestInstall,
}

/// Individual operation within a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOperation {
    pub id: String,
    pub operation_type: TransactionType,
    pub package_name: String,
    pub box_type: String,
    pub version_before: Option<String>,
    pub version_after: Option<String>,
    pub status: OperationStatus,
    pub error_message: Option<String>,
    pub rollback_data: Option<RollbackData>,
}

/// Status of an individual operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// Data needed to rollback an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackData {
    pub snapshot_id: Option<String>,
    pub previous_packages: Vec<String>,
    pub config_files: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

/// Complete transaction with all operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub operations: Vec<TransactionOperation>,
    pub status: TransactionStatus,
    pub created_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub rollback_snapshot_id: Option<String>,
    pub description: String,
}

/// Overall transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Planning,
    InProgress,
    Completed,
    PartiallyCompleted,
    Failed,
    RolledBack,
}

/// Transaction manager for atomic operations
pub struct TransactionManager {
    db: Database,
    snapshot_manager: SnapshotManager,
    recovery_manager: RecoveryManager,
    current_transaction: Option<Transaction>,
}

impl TransactionManager {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        let snapshot_manager = SnapshotManager::new().await?;
        let recovery_manager = RecoveryManager::new();
        
        Ok(Self {
            db,
            snapshot_manager,
            recovery_manager,
            current_transaction: None,
        })
    }
    
    /// Begin a new transaction
    pub async fn begin_transaction(
        &mut self,
        transaction_type: TransactionType,
        description: String,
    ) -> Result<String> {
        if self.current_transaction.is_some() {
            return Err(anyhow!("Transaction already in progress"));
        }
        
        let transaction_id = Uuid::new_v4().to_string();
        info!("Beginning transaction: {} - {}", transaction_id, description);
        
        // Create rollback snapshot
        let snapshot_id = self.snapshot_manager
            .create_snapshot(
                &format!("pre-transaction-{}", transaction_id),
                Some(&format!("Before {}", description))
            )
            .await?;
        
        let transaction = Transaction {
            id: transaction_id.clone(),
            transaction_type,
            operations: Vec::new(),
            status: TransactionStatus::Planning,
            created_at: SystemTime::now(),
            completed_at: None,
            rollback_snapshot_id: Some(snapshot_id),
            description,
        };
        
        // Save transaction to database
        self.save_transaction(&transaction).await?;
        self.current_transaction = Some(transaction);
        
        Ok(transaction_id)
    }
    
    /// Add an operation to the current transaction
    pub async fn add_operation(
        &mut self,
        operation_type: TransactionType,
        package_name: String,
        box_type: String,
        version_before: Option<String>,
    ) -> Result<String> {
        let transaction = self.current_transaction.as_mut()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        
        if transaction.status != TransactionStatus::Planning {
            return Err(anyhow!("Cannot add operations to non-planning transaction"));
        }
        
        let operation_id = Uuid::new_v4().to_string();
        
        let operation = TransactionOperation {
            id: operation_id.clone(),
            operation_type,
            package_name: package_name.clone(),
            box_type: box_type.clone(),
            version_before,
            version_after: None,
            status: OperationStatus::Pending,
            error_message: None,
            rollback_data: Some(self.prepare_rollback_data(&package_name, &box_type).await?),
        };
        
        transaction.operations.push(operation);
        self.save_transaction(transaction).await?;
        
        info!("Added operation {} to transaction {}", operation_id, transaction.id);
        Ok(operation_id)
    }
    
    /// Execute all operations in the current transaction
    pub async fn commit_transaction(&mut self) -> Result<TransactionResult> {
        let transaction = self.current_transaction.as_mut()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        
        if transaction.status != TransactionStatus::Planning {
            return Err(anyhow!("Transaction not in planning state"));
        }
        
        info!("Committing transaction: {}", transaction.id);
        transaction.status = TransactionStatus::InProgress;
        self.save_transaction(transaction).await?;
        
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut errors = Vec::new();
        
        // Execute operations in order
        for operation in &mut transaction.operations {
            operation.status = OperationStatus::InProgress;
            self.save_transaction(transaction).await?;
            
            match self.execute_operation(operation).await {
                Ok(version_after) => {
                    operation.version_after = version_after;
                    operation.status = OperationStatus::Completed;
                    successful_operations += 1;
                    info!("✅ Operation {} completed successfully", operation.id);
                }
                Err(e) => {
                    operation.status = OperationStatus::Failed;
                    operation.error_message = Some(e.to_string());
                    failed_operations += 1;
                    errors.push(e.to_string());
                    error!("❌ Operation {} failed: {}", operation.id, e);
                    
                    // Stop on first failure and rollback
                    break;
                }
            }
            
            self.save_transaction(transaction).await?;
        }
        
        // Determine final transaction status
        if failed_operations > 0 {
            warn!("Transaction failed with {} errors. Starting rollback...", failed_operations);
            transaction.status = TransactionStatus::Failed;
            
            // Attempt rollback
            if let Err(rollback_error) = self.rollback_transaction().await {
                error!("Rollback failed: {}", rollback_error);
                transaction.status = TransactionStatus::Failed; // Keep as failed
                errors.push(format!("Rollback failed: {}", rollback_error));
            } else {
                transaction.status = TransactionStatus::RolledBack;
            }
        } else if successful_operations == transaction.operations.len() {
            transaction.status = TransactionStatus::Completed;
            info!("✅ Transaction completed successfully");
        } else {
            transaction.status = TransactionStatus::PartiallyCompleted;
            warn!("Transaction partially completed");
        }
        
        transaction.completed_at = Some(SystemTime::now());
        self.save_transaction(transaction).await?;
        
        let result = TransactionResult {
            transaction_id: transaction.id.clone(),
            status: transaction.status.clone(),
            successful_operations,
            failed_operations,
            errors,
        };
        
        // Clear current transaction
        self.current_transaction = None;
        
        Ok(result)
    }
    
    /// Rollback the current transaction
    pub async fn rollback_transaction(&mut self) -> Result<()> {
        let transaction = self.current_transaction.as_mut()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        
        info!("Rolling back transaction: {}", transaction.id);
        
        // Rollback completed operations in reverse order
        let completed_operations: Vec<_> = transaction.operations
            .iter_mut()
            .filter(|op| op.status == OperationStatus::Completed)
            .collect();
        
        for operation in completed_operations.into_iter().rev() {
            match self.rollback_operation(operation).await {
                Ok(_) => {
                    operation.status = OperationStatus::RolledBack;
                    info!("✅ Rolled back operation: {}", operation.id);
                }
                Err(e) => {
                    error!("❌ Failed to rollback operation {}: {}", operation.id, e);
                    // Continue with other rollbacks
                }
            }
        }
        
        // Use snapshot rollback as final fallback
        if let Some(snapshot_id) = &transaction.rollback_snapshot_id {
            info!("Performing snapshot rollback to: {}", snapshot_id);
            if let Err(e) = self.snapshot_manager.revert_to_snapshot(snapshot_id).await {
                error!("Snapshot rollback failed: {}", e);
                return Err(anyhow!("Snapshot rollback failed: {}", e));
            }
        }
        
        transaction.status = TransactionStatus::RolledBack;
        self.save_transaction(transaction).await?;
        
        Ok(())
    }
    
    /// Abort the current transaction without execution
    pub async fn abort_transaction(&mut self) -> Result<()> {
        if let Some(transaction) = &mut self.current_transaction {
            info!("Aborting transaction: {}", transaction.id);
            transaction.status = TransactionStatus::Failed;
            self.save_transaction(transaction).await?;
            
            // Clean up snapshot if it exists
            if let Some(snapshot_id) = &transaction.rollback_snapshot_id {
                let _ = self.snapshot_manager.delete_snapshot(snapshot_id).await;
            }
        }
        
        self.current_transaction = None;
        Ok(())
    }
    
    async fn execute_operation(&self, operation: &TransactionOperation) -> Result<Option<String>> {
        // This would integrate with the secure executor from the boxes
        match operation.operation_type {
            TransactionType::Install => {
                // Use the appropriate box manager to install
                self.execute_install_operation(operation).await
            }
            TransactionType::Remove => {
                self.execute_remove_operation(operation).await
            }
            TransactionType::Update => {
                self.execute_update_operation(operation).await
            }
            TransactionType::ManifestInstall => {
                self.execute_install_operation(operation).await
            }
        }
    }
    
    async fn execute_install_operation(&self, operation: &TransactionOperation) -> Result<Option<String>> {
        // This would use the secure box managers
        info!("Executing install operation for {}", operation.package_name);
        
        // For now, return a placeholder - in real implementation this would
        // call the appropriate secure box manager
        Ok(Some("1.0.0".to_string()))
    }
    
    async fn execute_remove_operation(&self, operation: &TransactionOperation) -> Result<Option<String>> {
        info!("Executing remove operation for {}", operation.package_name);
        Ok(None)
    }
    
    async fn execute_update_operation(&self, operation: &TransactionOperation) -> Result<Option<String>> {
        info!("Executing update operation for {}", operation.package_name);
        Ok(Some("1.1.0".to_string()))
    }
    
    async fn rollback_operation(&self, operation: &TransactionOperation) -> Result<()> {
        info!("Rolling back operation: {} for {}", operation.operation_type, operation.package_name);
        
        match operation.operation_type {
            TransactionType::Install => {
                // Remove the package that was installed
                self.rollback_install_operation(operation).await
            }
            TransactionType::Remove => {
                // Reinstall the package that was removed
                self.rollback_remove_operation(operation).await
            }
            TransactionType::Update => {
                // Downgrade to previous version
                self.rollback_update_operation(operation).await
            }
            TransactionType::ManifestInstall => {
                self.rollback_install_operation(operation).await
            }
        }
    }
    
    async fn rollback_install_operation(&self, operation: &TransactionOperation) -> Result<()> {
        info!("Rolling back install of {}", operation.package_name);
        // Use the secure executor to remove the package
        Ok(())
    }
    
    async fn rollback_remove_operation(&self, operation: &TransactionOperation) -> Result<()> {
        info!("Rolling back removal of {}", operation.package_name);
        // Use the secure executor to reinstall the package
        Ok(())
    }
    
    async fn rollback_update_operation(&self, operation: &TransactionOperation) -> Result<()> {
        info!("Rolling back update of {}", operation.package_name);
        // Use the secure executor to downgrade the package
        Ok(())
    }
    
    async fn prepare_rollback_data(&self, package_name: &str, box_type: &str) -> Result<RollbackData> {
        // Gather information needed for rollback
        let mut rollback_data = RollbackData {
            snapshot_id: None,
            previous_packages: Vec::new(),
            config_files: HashMap::new(),
            dependencies: Vec::new(),
        };
        
        // Get current package version if installed
        if let Ok(installed) = self.db.get_installed_packages().await {
            if let Some(record) = installed.iter().find(|r| r.package_name == package_name) {
                rollback_data.previous_packages.push(record.package_name.clone());
            }
        }
        
        // Store important config files
        let config_paths = self.get_package_config_files(package_name, box_type).await?;
        for config_path in config_paths {
            if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
                rollback_data.config_files.insert(config_path, content);
            }
        }
        
        Ok(rollback_data)
    }
    
    async fn get_package_config_files(&self, _package_name: &str, _box_type: &str) -> Result<Vec<String>> {
        // This would identify important config files for the package
        // Implementation would vary by package manager
        Ok(vec![])
    }
    
    async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Save transaction state to database for persistence
        let json_data = serde_json::to_string(transaction)?;
        // In a real implementation, this would save to the database
        tokio::fs::write(
            format!("/tmp/omni_transaction_{}.json", transaction.id),
            json_data
        ).await?;
        Ok(())
    }
    
    /// Get transaction history
    pub async fn get_transaction_history(&self, limit: Option<usize>) -> Result<Vec<Transaction>> {
        // Load recent transactions from database
        // For now, return empty list
        Ok(vec![])
    }
    
    /// Get details of a specific transaction
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Option<Transaction>> {
        // Load specific transaction from database
        let file_path = format!("/tmp/omni_transaction_{}.json", transaction_id);
        
        if tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
            let json_data = tokio::fs::read_to_string(file_path).await?;
            let transaction: Transaction = serde_json::from_str(&json_data)?;
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }
}

/// Result of a transaction execution
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub errors: Vec<String>,
}

impl TransactionResult {
    pub fn is_successful(&self) -> bool {
        matches!(self.status, TransactionStatus::Completed)
    }
    
    pub fn is_rolled_back(&self) -> bool {
        matches!(self.status, TransactionStatus::RolledBack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let mut tm = TransactionManager::new().await.unwrap();
        
        // Begin transaction
        let tx_id = tm.begin_transaction(
            TransactionType::Install,
            "Test transaction".to_string()
        ).await.unwrap();
        
        assert!(tm.current_transaction.is_some());
        
        // Add operation
        let op_id = tm.add_operation(
            TransactionType::Install,
            "test-package".to_string(),
            "apt".to_string(),
            None
        ).await.unwrap();
        
        assert!(!op_id.is_empty());
        
        // Abort transaction
        tm.abort_transaction().await.unwrap();
        assert!(tm.current_transaction.is_none());
    }
    
    #[tokio::test]
    async fn test_transaction_persistence() {
        let mut tm = TransactionManager::new().await.unwrap();
        
        let tx_id = tm.begin_transaction(
            TransactionType::Install,
            "Persistence test".to_string()
        ).await.unwrap();
        
        // Transaction should be retrievable
        let retrieved = tm.get_transaction(&tx_id).await.unwrap();
        assert!(retrieved.is_some());
        
        tm.abort_transaction().await.unwrap();
    }
}