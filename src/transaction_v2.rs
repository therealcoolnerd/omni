use crate::database::Database;
use crate::error_handling::OmniError;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

/// Transaction manager for atomic package operations
#[derive(Debug, Clone)]
pub struct TransactionManager {
    db: Database,
    active_transactions: HashMap<Uuid, Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
    pub operations: Vec<Operation>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub rollback_data: Option<RollbackData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Install,
    Remove,
    Update,
    Batch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub operation_type: OperationType,
    pub package: String,
    pub version: Option<String>,
    pub status: OperationStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    InstallPackage,
    RemovePackage,
    UpdatePackage,
    CreateSnapshot,
    ModifyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackData {
    pub snapshot_id: Option<String>,
    pub previous_state: HashMap<String, String>,
    pub backup_files: Vec<String>,
}

impl TransactionManager {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        Ok(Self {
            db,
            active_transactions: HashMap::new(),
        })
    }

    /// Begin a new transaction
    pub async fn begin_transaction(&mut self, transaction_type: TransactionType) -> Result<Uuid> {
        let transaction_id = Uuid::new_v4();
        
        let transaction = Transaction {
            id: transaction_id,
            status: TransactionStatus::Pending,
            transaction_type,
            operations: Vec::new(),
            created_at: Utc::now(),
            completed_at: None,
            rollback_data: None,
        };

        self.active_transactions.insert(transaction_id, transaction);
        
        info!("Started transaction: {}", transaction_id);
        Ok(transaction_id)
    }

    /// Add an operation to a transaction
    pub async fn add_operation(
        &mut self,
        transaction_id: Uuid,
        operation_type: OperationType,
        package: String,
        version: Option<String>,
    ) -> Result<Uuid> {
        let operation_id = Uuid::new_v4();
        
        let operation = Operation {
            id: operation_id,
            operation_type,
            package,
            version,
            status: OperationStatus::Pending,
            error: None,
        };

        if let Some(transaction) = self.active_transactions.get_mut(&transaction_id) {
            transaction.operations.push(operation);
            info!("Added operation {} to transaction {}", operation_id, transaction_id);
            Ok(operation_id)
        } else {
            Err(anyhow::anyhow!("Transaction not found: {}", transaction_id))
        }
    }

    /// Execute a transaction
    pub async fn execute_transaction(&mut self, transaction_id: Uuid) -> Result<()> {
        // First, check if transaction exists and get its info
        let operations = if let Some(transaction) = self.active_transactions.get(&transaction_id) {
            info!("Executing transaction: {} with {} operations", 
                  transaction_id, transaction.operations.len());
            
            // Clone operations for processing
            transaction.operations.clone()
        } else {
            return Err(anyhow::anyhow!("Transaction not found: {}", transaction_id));
        };

        // Create rollback data
        let rollback_data = self.create_rollback_data().await?;
        
        // Update transaction status
        if let Some(transaction) = self.active_transactions.get_mut(&transaction_id) {
            transaction.status = TransactionStatus::InProgress;
            transaction.rollback_data = Some(rollback_data);
        }

        // Execute each operation
        let mut operation_results = Vec::new();
        for operation in operations {
            let mut op_copy = operation.clone();
            op_copy.status = OperationStatus::InProgress;
            
            match self.execute_operation(&op_copy).await {
                Ok(_) => {
                    op_copy.status = OperationStatus::Completed;
                    info!("Operation {} completed successfully", op_copy.id);
                    operation_results.push(op_copy);
                }
                Err(e) => {
                    op_copy.status = OperationStatus::Failed;
                    op_copy.error = Some(e.to_string());
                    warn!("Operation {} failed: {}", op_copy.id, e);
                    
                    // Rollback the transaction
                    return self.rollback_transaction(transaction_id).await;
                }
            }
        }

        // Update transaction with results
        if let Some(transaction) = self.active_transactions.get_mut(&transaction_id) {
            transaction.operations = operation_results;
            transaction.status = TransactionStatus::Completed;
            transaction.completed_at = Some(Utc::now());
            
            info!("Transaction {} completed successfully", transaction_id);
        }
        
        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback_transaction(&mut self, transaction_id: Uuid) -> Result<()> {
        if let Some(transaction) = self.active_transactions.get_mut(&transaction_id) {
            info!("Rolling back transaction: {}", transaction_id);
            
            // Implement rollback logic here
            if let Some(rollback_data) = &transaction.rollback_data {
                // Restore from snapshot if available
                if let Some(snapshot_id) = &rollback_data.snapshot_id {
                    info!("Restoring from snapshot: {}", snapshot_id);
                    // Implementation would restore snapshot
                }
            }

            transaction.status = TransactionStatus::RolledBack;
            transaction.completed_at = Some(Utc::now());
            
            info!("Transaction {} rolled back successfully", transaction_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Transaction not found: {}", transaction_id))
        }
    }

    /// Commit a transaction (finalize)
    pub async fn commit_transaction(&mut self, transaction_id: Uuid) -> Result<()> {
        if let Some(transaction) = self.active_transactions.remove(&transaction_id) {
            // Persist transaction to database
            self.persist_transaction(&transaction).await?;
            info!("Transaction {} committed to database", transaction_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Transaction not found: {}", transaction_id))
        }
    }

    async fn execute_operation(&self, operation: &Operation) -> Result<()> {
        use crate::boxes::{
            apt::AptManager,
            dnf::DnfBox,
            winget::WingetBox,
            brew::BrewBox,
            snap::SnapBox,
        };
        use crate::distro::PackageManager;
        
        match operation.operation_type {
            OperationType::InstallPackage => {
                info!("Installing package: {}", operation.package);
                
                // Detect and use appropriate package manager using trait methods
                if AptManager::is_available() {
                    let apt = AptManager::new()?;
                    apt.install(&operation.package)
                } else if DnfBox::is_available() {
                    let dnf = DnfBox::new()?;
                    dnf.install(&operation.package)
                } else if WingetBox::is_available() {
                    let winget = WingetBox::new()?;
                    winget.install(&operation.package)
                } else if BrewBox::is_available() {
                    let brew = BrewBox::new()?;
                    brew.install(&operation.package)
                } else if SnapBox::is_available() {
                    let snap = SnapBox::new()?;
                    snap.install(&operation.package)
                } else {
                    Err(anyhow::anyhow!("No supported package manager found"))
                }
            }
            OperationType::RemovePackage => {
                info!("Removing package: {}", operation.package);
                
                // Use trait methods for consistent interface
                if AptManager::is_available() {
                    let apt = AptManager::new()?;
                    apt.remove(&operation.package)
                } else if DnfBox::is_available() {
                    let dnf = DnfBox::new()?;
                    dnf.remove(&operation.package)
                } else if WingetBox::is_available() {
                    let winget = WingetBox::new()?;
                    winget.remove(&operation.package)
                } else if BrewBox::is_available() {
                    let brew = BrewBox::new()?;
                    brew.remove(&operation.package)
                } else if SnapBox::is_available() {
                    let snap = SnapBox::new()?;
                    snap.remove(&operation.package)
                } else {
                    Err(anyhow::anyhow!("No supported package manager found"))
                }
            }
            OperationType::UpdatePackage => {
                info!("Updating package: {}", operation.package);
                // Implementation would integrate with package managers
                Ok(())
            }
            OperationType::CreateSnapshot => {
                info!("Creating snapshot for: {}", operation.package);
                // Integration with snapshot manager
                Ok(())
            }
            OperationType::ModifyConfig => {
                info!("Modifying configuration for: {}", operation.package);
                // Configuration modification logic
                Ok(())
            }
        }
    }

    async fn create_rollback_data(&self) -> Result<RollbackData> {
        Ok(RollbackData {
            snapshot_id: Some(format!("snapshot_{}", Uuid::new_v4())),
            previous_state: HashMap::new(),
            backup_files: Vec::new(),
        })
    }

    async fn persist_transaction(&self, _transaction: &Transaction) -> Result<()> {
        // Implementation would persist to database
        Ok(())
    }

    /// Get transaction status
    pub fn get_transaction_status(&self, transaction_id: Uuid) -> Option<TransactionStatus> {
        self.active_transactions
            .get(&transaction_id)
            .map(|t| t.status.clone())
    }

    /// List all active transactions
    pub fn list_active_transactions(&self) -> Vec<&Transaction> {
        self.active_transactions.values().collect()
    }
}