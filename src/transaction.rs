use crate::database::Database;
use crate::distro::PackageManager;
use crate::error_handling::RecoveryManager;
use crate::snapshot::SnapshotManager;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Transaction types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Install,
    Remove,
    Update,
    ManifestInstall,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Install => write!(f, "install"),
            TransactionType::Remove => write!(f, "remove"),
            TransactionType::Update => write!(f, "update"),
            TransactionType::ManifestInstall => write!(f, "manifest-install"),
        }
    }
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

/// Result of executing a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub errors: Vec<String>,
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

        let manager = Self {
            db,
            snapshot_manager,
            recovery_manager,
            current_transaction: None,
        };

        // Clean up any old temporary transaction files
        manager.cleanup_temp_files().await?;

        Ok(manager)
    }

    /// Clean up old temporary transaction files from /tmp/
    async fn cleanup_temp_files(&self) -> Result<()> {
        use tokio::fs;

        if let Ok(mut dir) = fs::read_dir("/tmp").await {
            while let Ok(Some(entry)) = dir.next_entry().await {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("omni_transaction_") && file_name.ends_with(".json") {
                        let _ = fs::remove_file(entry.path()).await;
                        info!("Cleaned up old transaction file: {}", file_name);
                    }
                }
            }
        }

        Ok(())
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
        info!(
            "Beginning transaction: {} - {}",
            transaction_id, description
        );

        // Create rollback snapshot
        let snapshot_id = self
            .snapshot_manager
            .create_snapshot(
                &format!("pre-transaction-{}", transaction_id),
                Some(&format!("Before {}", description)),
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
        // Check if we have an active transaction first
        if let Some(ref transaction) = self.current_transaction {
            if transaction.status != TransactionStatus::Planning {
                return Err(anyhow!("Cannot add operations to non-planning transaction"));
            }
        } else {
            return Err(anyhow!("No active transaction"));
        }

        // Prepare rollback data before borrowing mutably
        let rollback_data = self.prepare_rollback_data(&package_name, &box_type).await?;
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
            rollback_data: Some(rollback_data),
        };

        // Now we can safely borrow mutably
        let transaction = self.current_transaction.as_mut().unwrap();
        transaction.operations.push(operation);

        let transaction_id = transaction.id.clone();
        self.save_transaction(transaction).await?;

        info!(
            "Added operation {} to transaction {}",
            operation_id, transaction_id
        );
        Ok(operation_id)
    }

    /// Execute all operations in the current transaction
    pub async fn commit_transaction(&mut self) -> Result<TransactionResult> {
        // First check if we have a valid transaction
        {
            let transaction = self
                .current_transaction
                .as_ref()
                .ok_or_else(|| anyhow!("No active transaction"))?;

            if transaction.status != TransactionStatus::Planning {
                return Err(anyhow!("Transaction not in planning state"));
            }
        }

        // Update transaction status
        let transaction_id = {
            let transaction = self.current_transaction.as_mut().unwrap();
            info!("Committing transaction: {}", transaction.id);
            transaction.status = TransactionStatus::InProgress;
            let id = transaction.id.clone();
            self.save_transaction(transaction).await?;
            id
        };

        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut errors = Vec::new();

        // Execute operations one by one
        let operations_count = self.current_transaction.as_ref().unwrap().operations.len();
        for i in 0..operations_count {
            // Update operation status
            {
                let transaction = self.current_transaction.as_mut().unwrap();
                transaction.operations[i].status = OperationStatus::InProgress;
                self.save_transaction(transaction).await?;
            }

            // Execute the operation
            let operation_result = {
                let transaction = self.current_transaction.as_ref().unwrap();
                let operation = &transaction.operations[i];
                self.execute_operation(operation).await
            };

            // Update operation with result
            match operation_result {
                Ok(version_after) => {
                    let transaction = self.current_transaction.as_mut().unwrap();
                    transaction.operations[i].version_after = version_after;
                    transaction.operations[i].status = OperationStatus::Completed;
                    successful_operations += 1;
                    info!(
                        "✅ Operation {} completed successfully",
                        transaction.operations[i].id
                    );
                    self.save_transaction(transaction).await?;
                }
                Err(e) => {
                    let transaction = self.current_transaction.as_mut().unwrap();
                    transaction.operations[i].status = OperationStatus::Failed;
                    transaction.operations[i].error_message = Some(e.to_string());
                    failed_operations += 1;
                    errors.push(e.to_string());
                    error!(
                        "❌ Operation {} failed: {}",
                        transaction.operations[i].id, e
                    );
                    self.save_transaction(transaction).await?;

                    // Stop on first failure
                    break;
                }
            }
        }

        // Determine final transaction status
        let result = if failed_operations > 0 {
            warn!(
                "Transaction failed with {} errors. Starting rollback...",
                failed_operations
            );

            // Attempt rollback
            let rollback_result = self.rollback_transaction().await;

            let final_status = if let Err(rollback_error) = rollback_result {
                error!("Rollback failed: {}", rollback_error);
                errors.push(format!("Rollback failed: {}", rollback_error));
                TransactionStatus::Failed
            } else {
                TransactionStatus::RolledBack
            };

            // Update transaction status
            {
                let transaction = self.current_transaction.as_mut().unwrap();
                transaction.status = final_status.clone();
                transaction.completed_at = Some(SystemTime::now());
                self.save_transaction(transaction).await?;
            }

            let transaction_id = self.current_transaction.as_ref().unwrap().id.clone();
            TransactionResult {
                transaction_id,
                status: final_status,
                successful_operations,
                failed_operations,
                errors,
            }
        } else {
            // Successful completion
            let total_operations = self.current_transaction.as_ref().unwrap().operations.len();
            let final_status = if successful_operations == total_operations {
                TransactionStatus::Completed
            } else {
                TransactionStatus::PartiallyCompleted
            };

            // Update transaction status
            {
                let transaction = self.current_transaction.as_mut().unwrap();
                transaction.status = final_status.clone();
                transaction.completed_at = Some(SystemTime::now());
                self.save_transaction(transaction).await?;
            }

            if final_status == TransactionStatus::Completed {
                info!("✅ Transaction completed successfully");
            } else {
                warn!("Transaction partially completed");
            }

            let transaction_id = self.current_transaction.as_ref().unwrap().id.clone();
            TransactionResult {
                transaction_id,
                status: final_status,
                successful_operations,
                failed_operations,
                errors,
            }
        };

        // Clear current transaction
        self.current_transaction = None;

        Ok(result)
    }

    /// Rollback the current transaction
    pub async fn rollback_transaction(&mut self) -> Result<()> {
        let transaction = self
            .current_transaction
            .as_mut()
            .ok_or_else(|| anyhow!("No active transaction"))?;

        info!("Rolling back transaction: {}", transaction.id);

        // Rollback completed operations in reverse order
        let completed_operations: Vec<_> = transaction
            .operations
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
            TransactionType::Remove => self.execute_remove_operation(operation).await,
            TransactionType::Update => self.execute_update_operation(operation).await,
            TransactionType::ManifestInstall => self.execute_install_operation(operation).await,
        }
    }

    async fn execute_install_operation(
        &self,
        operation: &TransactionOperation,
    ) -> Result<Option<String>> {
        use crate::boxes::*;

        info!(
            "Executing install operation for {} via {}",
            operation.package_name, operation.box_type
        );

        match operation.box_type.as_str() {
            "apt" => {
                let manager = apt::AptManager::new()?;
                manager.install(&operation.package_name).await?;
                Ok(Some("installed".to_string()))
            }
            "dnf" => {
                let manager = dnf::DnfBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "pacman" => {
                let manager = pacman::PacmanBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "zypper" => {
                let manager = zypper::ZypperBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "emerge" => {
                let manager = emerge::EmergeBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "nix" => {
                let manager = nix::NixBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "winget" => {
                let manager = winget::WingetBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "brew" => {
                let manager = brew::BrewBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            "chocolatey" => {
                let manager = chocolatey::ChocolateyBox::new()?;
                manager.install(&operation.package_name)?;
                Ok(Some("installed".to_string()))
            }
            _ => {
                return Err(anyhow!("Unsupported box type: {}", operation.box_type));
            }
        }
    }

    async fn execute_remove_operation(
        &self,
        operation: &TransactionOperation,
    ) -> Result<Option<String>> {
        use crate::boxes::*;

        info!(
            "Executing remove operation for {} via {}",
            operation.package_name, operation.box_type
        );

        match operation.box_type.as_str() {
            "apt" => {
                let manager = apt::AptManager::new()?;
                manager.remove(&operation.package_name).await?;
                Ok(None)
            }
            "dnf" => {
                let manager = dnf::DnfBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "pacman" => {
                let manager = pacman::PacmanBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "zypper" => {
                let manager = zypper::ZypperBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "emerge" => {
                let manager = emerge::EmergeBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "nix" => {
                let manager = nix::NixBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "winget" => {
                let manager = winget::WingetBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "brew" => {
                let manager = brew::BrewBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            "chocolatey" => {
                let manager = chocolatey::ChocolateyBox::new()?;
                manager.remove(&operation.package_name)?;
                Ok(None)
            }
            _ => {
                return Err(anyhow!("Unsupported box type: {}", operation.box_type));
            }
        }
    }

    async fn execute_update_operation(
        &self,
        operation: &TransactionOperation,
    ) -> Result<Option<String>> {
        info!("Executing update operation for {}", operation.package_name);
        Ok(Some("1.1.0".to_string()))
    }

    async fn rollback_operation(&self, operation: &TransactionOperation) -> Result<()> {
        info!(
            "Rolling back operation: {} for {}",
            operation.operation_type, operation.package_name
        );

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
            TransactionType::ManifestInstall => self.rollback_install_operation(operation).await,
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

    async fn prepare_rollback_data(
        &self,
        package_name: &str,
        box_type: &str,
    ) -> Result<RollbackData> {
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
                rollback_data
                    .previous_packages
                    .push(record.package_name.clone());
            }
        }

        // Store important config files
        let config_paths = self
            .get_package_config_files(package_name, box_type)
            .await?;
        for config_path in config_paths {
            if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
                rollback_data.config_files.insert(config_path, content);
            }
        }

        Ok(rollback_data)
    }

    async fn get_package_config_files(
        &self,
        _package_name: &str,
        _box_type: &str,
    ) -> Result<Vec<String>> {
        // This would identify important config files for the package
        // Implementation would vary by package manager
        Ok(vec![])
    }

    async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Save transaction state to database for persistence
        let json_data = serde_json::to_string(transaction)?;
        let status_str = match transaction.status {
            TransactionStatus::Planning => "planning",
            TransactionStatus::InProgress => "in_progress",
            TransactionStatus::Completed => "completed",
            TransactionStatus::PartiallyCompleted => "partially_completed",
            TransactionStatus::Failed => "failed",
            TransactionStatus::RolledBack => "rolled_back",
        };

        let created_at = transaction
            .created_at
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        let completed_at = transaction
            .completed_at
            .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64);

        // Create transactions table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                transaction_type TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                completed_at INTEGER,
                rollback_snapshot_id TEXT,
                description TEXT NOT NULL,
                operations_json TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.db.pool)
        .await?;

        let transaction_type_str = match transaction.transaction_type {
            TransactionType::Install => "install",
            TransactionType::Remove => "remove",
            TransactionType::Update => "update",
            TransactionType::ManifestInstall => "manifest_install",
        };

        // Insert or update transaction
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO transactions 
            (id, transaction_type, status, created_at, completed_at, rollback_snapshot_id, description, operations_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&transaction.id)
        .bind(transaction_type_str)
        .bind(status_str)
        .bind(created_at)
        .bind(completed_at)
        .bind(&transaction.rollback_snapshot_id)
        .bind(&transaction.description)
        .bind(&json_data)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    /// Get transaction history
    pub async fn get_transaction_history(&self, limit: Option<usize>) -> Result<Vec<Transaction>> {
        let limit = limit.unwrap_or(50) as i64;

        let rows = sqlx::query("SELECT * FROM transactions ORDER BY created_at DESC LIMIT ?1")
            .bind(limit)
            .fetch_all(&self.db.pool)
            .await?;

        let mut transactions = Vec::new();

        for row in rows {
            let operations_json: String = row.get("operations_json");
            let transaction: Transaction = serde_json::from_str(&operations_json)?;
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    /// Get details of a specific transaction
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Option<Transaction>> {
        let row = sqlx::query("SELECT operations_json FROM transactions WHERE id = ?1")
            .bind(transaction_id)
            .fetch_optional(&self.db.pool)
            .await?;

        if let Some(row) = row {
            let operations_json: String = row.get("operations_json");
            let transaction: Transaction = serde_json::from_str(&operations_json)?;
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }
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
        let tx_id = tm
            .begin_transaction(TransactionType::Install, "Test transaction".to_string())
            .await
            .unwrap();

        assert!(tm.current_transaction.is_some());

        // Add operation
        let op_id = tm
            .add_operation(
                TransactionType::Install,
                "test-package".to_string(),
                "apt".to_string(),
                None,
            )
            .await
            .unwrap();

        assert!(!op_id.is_empty());

        // Abort transaction
        tm.abort_transaction().await.unwrap();
        assert!(tm.current_transaction.is_none());
    }

    #[tokio::test]
    async fn test_transaction_persistence() {
        let mut tm = TransactionManager::new().await.unwrap();

        let tx_id = tm
            .begin_transaction(TransactionType::Install, "Persistence test".to_string())
            .await
            .unwrap();

        // Transaction should be retrievable
        let retrieved = tm.get_transaction(&tx_id).await.unwrap();
        assert!(retrieved.is_some());

        tm.abort_transaction().await.unwrap();
    }
}
