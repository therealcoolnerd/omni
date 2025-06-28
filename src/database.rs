use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::OmniConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallRecord {
    pub id: String,
    pub package_name: String,
    pub box_type: String,
    pub version: Option<String>,
    pub source_url: Option<String>,
    pub install_path: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub status: InstallStatus,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstallStatus {
    Success,
    Failed,
    Removed,
    Updated,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub packages: Vec<InstallRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageCache {
    pub package_name: String,
    pub box_type: String,
    pub version: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub total_hits: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub total_install_records: usize,
    pub total_snapshots: usize,
    pub cache_stats: CacheStats,
    pub integrity_ok: bool,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let data_dir = OmniConfig::data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        
        let database_url = format!("sqlite:{}/omni.db", data_dir.display());
        
        // Configure connection pool for optimal performance
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .max_lifetime(Some(std::time::Duration::from_secs(3600))) // 1 hour
            .idle_timeout(Some(std::time::Duration::from_secs(600)))  // 10 minutes
            .test_before_acquire(true)
            .connect(&database_url)
            .await?;
        
        let db = Database { pool };
        db.migrate().await?;
        
        Ok(db)
    }
    
    /// Create an in-memory database for testing
    pub async fn new_in_memory() -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        
        let db = Database { pool };
        db.migrate().await?;
        
        Ok(db)
    }
    
    async fn migrate(&self) -> Result<()> {
        // Create tables with optimized schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS install_records (
                id TEXT PRIMARY KEY,
                package_name TEXT NOT NULL,
                box_type TEXT NOT NULL,
                version TEXT,
                source_url TEXT,
                install_path TEXT,
                installed_at TEXT NOT NULL,
                status TEXT NOT NULL,
                metadata TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshot_packages (
                snapshot_id TEXT NOT NULL,
                install_record_id TEXT NOT NULL,
                FOREIGN KEY (snapshot_id) REFERENCES snapshots (id),
                FOREIGN KEY (install_record_id) REFERENCES install_records (id)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS package_cache (
                package_name TEXT NOT NULL,
                box_type TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                dependencies TEXT,
                cached_at TEXT NOT NULL,
                expires_at TEXT,
                hits INTEGER DEFAULT 0,
                PRIMARY KEY (package_name, box_type)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create performance indexes
        self.create_indexes().await?;
        
        // Optimize database settings
        self.optimize_database().await?;

        Ok(())
    }
    
    async fn create_indexes(&self) -> Result<()> {
        // Index for install_records queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_install_records_package_name ON install_records(package_name)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_install_records_box_type ON install_records(box_type)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_install_records_status ON install_records(status)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_install_records_installed_at ON install_records(installed_at)")
            .execute(&self.pool).await?;
        
        // Composite index for common queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_install_records_package_box ON install_records(package_name, box_type)")
            .execute(&self.pool).await?;
        
        // Index for snapshot queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_created_at ON snapshots(created_at)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_name ON snapshots(name)")
            .execute(&self.pool).await?;
        
        // Index for snapshot_packages
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshot_packages_snapshot_id ON snapshot_packages(snapshot_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshot_packages_record_id ON snapshot_packages(install_record_id)")
            .execute(&self.pool).await?;
        
        // Index for package_cache queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_package_cache_cached_at ON package_cache(cached_at)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_package_cache_expires_at ON package_cache(expires_at)")
            .execute(&self.pool).await?;
        
        Ok(())
    }
    
    async fn optimize_database(&self) -> Result<()> {
        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&self.pool).await?;
        
        // Set synchronous to NORMAL for better performance
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.pool).await?;
        
        // Increase cache size (10MB)
        sqlx::query("PRAGMA cache_size = -10000")
            .execute(&self.pool).await?;
        
        // Enable memory-mapped I/O (256MB)
        sqlx::query("PRAGMA mmap_size = 268435456")
            .execute(&self.pool).await?;
        
        // Optimize temp store
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&self.pool).await?;
        
        // Set busy timeout (30 seconds)
        sqlx::query("PRAGMA busy_timeout = 30000")
            .execute(&self.pool).await?;
        
        Ok(())
    }
    
    pub async fn record_install(&self, record: &InstallRecord) -> Result<()> {
        let status_str = match record.status {
            InstallStatus::Success => "success",
            InstallStatus::Failed => "failed",
            InstallStatus::Removed => "removed",
            InstallStatus::Updated => "updated",
        };
        
        sqlx::query(
            r#"
            INSERT INTO install_records 
            (id, package_name, box_type, version, source_url, install_path, installed_at, status, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&record.id)
        .bind(&record.package_name)
        .bind(&record.box_type)
        .bind(&record.version)
        .bind(&record.source_url)
        .bind(&record.install_path)
        .bind(record.installed_at.to_rfc3339())
        .bind(status_str)
        .bind(&record.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_install_history(&self, limit: Option<i64>) -> Result<Vec<InstallRecord>> {
        let limit = limit.unwrap_or(100);
        
        let rows = sqlx::query(
            "SELECT * FROM install_records ORDER BY installed_at DESC LIMIT ?1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        
        let mut records = Vec::new();
        for row in rows {
            let status = match row.get::<String, _>("status").as_str() {
                "success" => InstallStatus::Success,
                "failed" => InstallStatus::Failed,
                "removed" => InstallStatus::Removed,
                "updated" => InstallStatus::Updated,
                _ => InstallStatus::Failed,
            };
            
            let installed_at: String = row.get("installed_at");
            let installed_at = DateTime::parse_from_rfc3339(&installed_at)?
                .with_timezone(&Utc);
            
            records.push(InstallRecord {
                id: row.get("id"),
                package_name: row.get("package_name"),
                box_type: row.get("box_type"),
                version: row.get("version"),
                source_url: row.get("source_url"),
                install_path: row.get("install_path"),
                installed_at,
                status,
                metadata: row.get("metadata"),
            });
        }
        
        Ok(records)
    }
    
    pub async fn get_installed_packages(&self) -> Result<Vec<InstallRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM install_records WHERE status = 'success' ORDER BY installed_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut records = Vec::new();
        for row in rows {
            let installed_at: String = row.get("installed_at");
            let installed_at = DateTime::parse_from_rfc3339(&installed_at)?
                .with_timezone(&Utc);
            
            records.push(InstallRecord {
                id: row.get("id"),
                package_name: row.get("package_name"),
                box_type: row.get("box_type"),
                version: row.get("version"),
                source_url: row.get("source_url"),
                install_path: row.get("install_path"),
                installed_at,
                status: InstallStatus::Success,
                metadata: row.get("metadata"),
            });
        }
        
        Ok(records)
    }
    
    pub async fn create_snapshot(&self, name: &str, description: Option<&str>) -> Result<String> {
        let snapshot_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        
        let installed_packages = self.get_installed_packages().await?;
        
        sqlx::query(
            "INSERT INTO snapshots (id, name, description, created_at) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&snapshot_id)
        .bind(name)
        .bind(description)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        for package in &installed_packages {
            sqlx::query(
                "INSERT INTO snapshot_packages (snapshot_id, install_record_id) VALUES (?1, ?2)"
            )
            .bind(&snapshot_id)
            .bind(&package.id)
            .execute(&self.pool)
            .await?;
        }
        
        Ok(snapshot_id)
    }
    
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let rows = sqlx::query("SELECT * FROM snapshots ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut snapshots = Vec::new();
        
        for row in rows {
            let snapshot_id: String = row.get("id");
            let created_at: String = row.get("created_at");
            let created_at = DateTime::parse_from_rfc3339(&created_at)?
                .with_timezone(&Utc);
            
            let packages = self.get_snapshot_packages(&snapshot_id).await?;
            
            snapshots.push(Snapshot {
                id: snapshot_id,
                name: row.get("name"),
                description: row.get("description"),
                created_at,
                packages,
            });
        }
        
        Ok(snapshots)
    }
    
    async fn get_snapshot_packages(&self, snapshot_id: &str) -> Result<Vec<InstallRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT ir.* FROM install_records ir
            JOIN snapshot_packages sp ON ir.id = sp.install_record_id
            WHERE sp.snapshot_id = ?1
            "#
        )
        .bind(snapshot_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut records = Vec::new();
        for row in rows {
            let installed_at: String = row.get("installed_at");
            let installed_at = DateTime::parse_from_rfc3339(&installed_at)?
                .with_timezone(&Utc);
            
            records.push(InstallRecord {
                id: row.get("id"),
                package_name: row.get("package_name"),
                box_type: row.get("box_type"),
                version: row.get("version"),
                source_url: row.get("source_url"),
                install_path: row.get("install_path"),
                installed_at,
                status: InstallStatus::Success,
                metadata: row.get("metadata"),
            });
        }
        
        Ok(records)
    }
    
    pub async fn cache_package_info(&self, cache_entry: &PackageCache) -> Result<()> {
        let dependencies_json = serde_json::to_string(&cache_entry.dependencies)?;
        
        // Calculate expiration time (default 24 hours)
        let expires_at = Utc::now() + chrono::Duration::hours(24);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO package_cache 
            (package_name, box_type, version, description, dependencies, cached_at, expires_at, hits)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, COALESCE((SELECT hits FROM package_cache WHERE package_name = ?1 AND box_type = ?2), 0))
            "#,
        )
        .bind(&cache_entry.package_name)
        .bind(&cache_entry.box_type)
        .bind(&cache_entry.version)
        .bind(&cache_entry.description)
        .bind(&dependencies_json)
        .bind(cache_entry.cached_at.to_rfc3339())
        .bind(expires_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_cached_package_info(&self, package_name: &str, box_type: &str) -> Result<Option<PackageCache>> {
        // First check if cache entry exists and is not expired
        let row = sqlx::query(
            r#"
            SELECT * FROM package_cache 
            WHERE package_name = ?1 AND box_type = ?2
            AND (expires_at IS NULL OR expires_at > datetime('now'))
            "#
        )
        .bind(package_name)
        .bind(box_type)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            // Increment hit counter
            sqlx::query("UPDATE package_cache SET hits = hits + 1 WHERE package_name = ?1 AND box_type = ?2")
                .bind(package_name)
                .bind(box_type)
                .execute(&self.pool)
                .await?;
            
            let cached_at: String = row.get("cached_at");
            let cached_at = DateTime::parse_from_rfc3339(&cached_at)?
                .with_timezone(&Utc);
            
            let dependencies_json: String = row.get("dependencies");
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_json)?;
            
            Ok(Some(PackageCache {
                package_name: row.get("package_name"),
                box_type: row.get("box_type"),
                version: row.get("version"),
                description: row.get("description"),
                dependencies,
                cached_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Clean expired cache entries
    pub async fn clean_expired_cache(&self) -> Result<usize> {
        let result = sqlx::query("DELETE FROM package_cache WHERE expires_at < datetime('now')")
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let total_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM package_cache")
            .fetch_one(&self.pool)
            .await?;
        
        let expired_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM package_cache WHERE expires_at < datetime('now')")
            .fetch_one(&self.pool)
            .await?;
        
        let total_hits: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(hits), 0) FROM package_cache")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        
        Ok(CacheStats {
            total_entries: total_entries as usize,
            expired_entries: expired_entries as usize,
            total_hits: total_hits as usize,
        })
    }
    
    /// Optimize database by running maintenance tasks
    pub async fn maintenance(&self) -> Result<()> {
        // Clean expired cache entries
        let cleaned = self.clean_expired_cache().await?;
        if cleaned > 0 {
            tracing::info!("Cleaned {} expired cache entries", cleaned);
        }
        
        // Analyze tables for query optimizer
        sqlx::query("ANALYZE").execute(&self.pool).await?;
        
        // Vacuum if needed (only if significant deletions occurred)
        if cleaned > 100 {
            sqlx::query("VACUUM").execute(&self.pool).await?;
            tracing::info!("Database vacuum completed");
        }
        
        Ok(())
    }
    
    /// Get database health information
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        let cache_stats = self.get_cache_stats().await?;
        
        let total_records: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM install_records")
            .fetch_one(&self.pool)
            .await?;
        
        let total_snapshots: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshots")
            .fetch_one(&self.pool)
            .await?;
        
        // Check database integrity
        let integrity_check: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(DatabaseHealth {
            total_install_records: total_records as usize,
            total_snapshots: total_snapshots as usize,
            cache_stats,
            integrity_ok: integrity_check == "ok",
        })
    }
}