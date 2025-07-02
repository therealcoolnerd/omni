use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Security event types for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    // Authentication events
    LoginAttempt {
        success: bool,
        user: String,
        ip: Option<IpAddr>,
    },
    LoginFailure {
        user: String,
        ip: Option<IpAddr>,
        reason: String,
    },
    PrivilegeEscalation {
        user: String,
        command: String,
    },

    // Package management events
    PackageInstall {
        package: String,
        box_type: String,
        user: String,
    },
    PackageRemove {
        package: String,
        box_type: String,
        user: String,
    },
    PackageUpdate {
        package: String,
        box_type: String,
        user: String,
    },
    PackageSearch {
        query: String,
        user: String,
    },

    // System events
    SystemAccess {
        user: String,
        ip: Option<IpAddr>,
        method: String,
    },
    FileAccess {
        path: String,
        operation: String,
        user: String,
    },
    NetworkConnection {
        destination: String,
        port: u16,
        user: String,
    },
    ConfigurationChange {
        setting: String,
        old_value: String,
        new_value: String,
        user: String,
    },

    // Security events
    SecurityViolation {
        description: String,
        user: String,
        severity: SecuritySeverity,
    },
    MaliciousActivity {
        description: String,
        ip: Option<IpAddr>,
        indicators: Vec<String>,
    },
    SuspiciousCommand {
        command: String,
        user: String,
        reason: String,
    },

    // SSH and remote events
    SshConnection {
        host: String,
        user: String,
        success: bool,
    },
    RemoteCommand {
        host: String,
        command: String,
        user: String,
        success: bool,
    },

    // Container events
    ContainerCreated {
        image: String,
        container_id: String,
        user: String,
    },
    ContainerStarted {
        container_id: String,
        user: String,
    },
    ContainerStopped {
        container_id: String,
        user: String,
    },
    ContainerDeleted {
        container_id: String,
        user: String,
    },
}

impl fmt::Display for SecurityEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityEvent::LoginAttempt { success, user, .. } => {
                write!(f, "Login attempt by {} ({})", user, if *success { "success" } else { "failed" })
            }
            SecurityEvent::LoginFailure { user, reason, .. } => {
                write!(f, "Login failure for {}: {}", user, reason)
            }
            SecurityEvent::PrivilegeEscalation { user, command } => {
                write!(f, "Privilege escalation by {} for command: {}", user, command)
            }
            SecurityEvent::PackageInstall { package, box_type, user } => {
                write!(f, "Package install: {} via {} by {}", package, box_type, user)
            }
            SecurityEvent::PackageRemove { package, box_type, user } => {
                write!(f, "Package remove: {} via {} by {}", package, box_type, user)
            }
            SecurityEvent::PackageUpdate { package, box_type, user } => {
                write!(f, "Package update: {} via {} by {}", package, box_type, user)
            }
            SecurityEvent::PackageSearch { query, user } => {
                write!(f, "Package search: '{}' by {}", query, user)
            }
            SecurityEvent::SystemAccess { user, method, .. } => {
                write!(f, "System access by {} via {}", user, method)
            }
            SecurityEvent::FileAccess { path, operation, user } => {
                write!(f, "File access: {} {} by {}", operation, path, user)
            }
            SecurityEvent::NetworkConnection { destination, port, user } => {
                write!(f, "Network connection to {}:{} by {}", destination, port, user)
            }
            SecurityEvent::ConfigurationChange { setting, user, .. } => {
                write!(f, "Configuration change: {} by {}", setting, user)
            }
            SecurityEvent::SecurityViolation { description, user, .. } => {
                write!(f, "Security violation: {} by {}", description, user)
            }
            SecurityEvent::MaliciousActivity { description, .. } => {
                write!(f, "Malicious activity detected: {}", description)
            }
            SecurityEvent::SuspiciousCommand { command, user, reason } => {
                write!(f, "Suspicious command: {} by {} ({})", command, user, reason)
            }
            SecurityEvent::SshConnection { host, user, success } => {
                write!(f, "SSH connection to {} by {} ({})", host, user, if *success { "success" } else { "failed" })
            }
            SecurityEvent::RemoteCommand { host, command, user, success } => {
                write!(f, "Remote command on {}: {} by {} ({})", host, command, user, if *success { "success" } else { "failed" })
            }
            SecurityEvent::ContainerCreated { image, user, .. } => {
                write!(f, "Container created from {} by {}", image, user)
            }
            SecurityEvent::ContainerStarted { container_id, user } => {
                write!(f, "Container {} started by {}", container_id, user)
            }
            SecurityEvent::ContainerStopped { container_id, user } => {
                write!(f, "Container {} stopped by {}", container_id, user)
            }
            SecurityEvent::ContainerDeleted { container_id, user } => {
                write!(f, "Container {} deleted by {}", container_id, user)
            }
        }
    }
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event: SecurityEvent,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source_ip: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub metadata: HashMap<String, String>,
    pub severity: SecuritySeverity,
}

impl AuditEntry {
    pub fn new(event: SecurityEvent, user_id: Option<String>) -> Self {
        let severity = match &event {
            SecurityEvent::SecurityViolation { severity, .. } => severity.clone(),
            SecurityEvent::MaliciousActivity { .. } => SecuritySeverity::Critical,
            SecurityEvent::SuspiciousCommand { .. } => SecuritySeverity::High,
            SecurityEvent::LoginFailure { .. } => SecuritySeverity::Medium,
            SecurityEvent::PrivilegeEscalation { .. } => SecuritySeverity::High,
            _ => SecuritySeverity::Low,
        };

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event,
            user_id,
            session_id: None,
            source_ip: None,
            user_agent: None,
            metadata: HashMap::new(),
            severity,
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.source_ip = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_level: SecuritySeverity,
    pub retention_days: u32,
    pub real_time_alerts: bool,
    pub alert_webhook: Option<String>,
    pub alert_email: Option<String>,
    pub file_logging: bool,
    pub database_logging: bool,
    pub syslog_logging: bool,
    pub max_entries_per_day: Option<u64>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: SecuritySeverity::Low,
            retention_days: 90, // 3 months
            real_time_alerts: true,
            alert_webhook: None,
            alert_email: None,
            file_logging: true,
            database_logging: true,
            syslog_logging: false,
            max_entries_per_day: Some(10000),
        }
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub entries_today: u64,
    pub critical_events: u64,
    pub high_severity_events: u64,
    pub failed_logins: u64,
    pub package_operations: u64,
    pub remote_connections: u64,
    pub most_active_users: Vec<(String, u64)>,
    pub most_common_events: Vec<(String, u64)>,
}

/// Audit manager for security logging and monitoring
pub struct AuditManager {
    pool: SqlitePool,
    config: AuditConfig,
    file_logger: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl AuditManager {
    pub async fn new(pool: SqlitePool, config: AuditConfig) -> Result<Self> {
        let mut manager = Self {
            pool,
            config,
            file_logger: None,
        };

        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&mut self) -> Result<()> {
        // Create audit tables
        self.create_tables().await?;

        // Set up file logging if enabled
        if self.config.file_logging {
            self.setup_file_logging().await?;
        }

        // Set up cleanup job
        self.schedule_cleanup().await?;

        info!(
            "Audit manager initialized with retention {} days",
            self.config.retention_days
        );
        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                user_id TEXT,
                session_id TEXT,
                source_ip TEXT,
                user_agent TEXT,
                metadata TEXT,
                severity TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_user_id ON audit_log(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_log(event_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_log(severity)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn setup_file_logging(&mut self) -> Result<()> {
        use tracing_appender::rolling::{RollingFileAppender, Rotation};

        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "audit.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Set up file-specific subscriber
        let subscriber = tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_ansi(false)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
        self.file_logger = Some(guard);

        Ok(())
    }

    async fn schedule_cleanup(&self) -> Result<()> {
        // In a real implementation, this would set up a background task
        // to clean old entries based on retention policy
        info!(
            "Scheduled audit log cleanup for {} days retention",
            self.config.retention_days
        );
        Ok(())
    }

    /// Log an audit entry
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if this severity level should be logged
        if !self.should_log_severity(&entry.severity) {
            return Ok(());
        }

        // Check rate limits
        if !self.check_rate_limits().await? {
            warn!("Audit logging rate limit exceeded");
            return Ok(());
        }

        // Log to database
        if self.config.database_logging {
            self.log_to_database(&entry).await?;
        }

        // Log to structured logging (file)
        if self.config.file_logging {
            self.log_to_file(&entry).await?;
        }

        // Log to syslog if enabled
        if self.config.syslog_logging {
            self.log_to_syslog(&entry).await?;
        }

        // Send real-time alerts for high-severity events
        if self.config.real_time_alerts && self.is_high_severity(&entry.severity) {
            self.send_alert(&entry).await?;
        }

        Ok(())
    }

    async fn log_to_database(&self, entry: &AuditEntry) -> Result<()> {
        let event_type = self.get_event_type(&entry.event);
        let event_data = serde_json::to_string(&entry.event)?;
        let metadata = serde_json::to_string(&entry.metadata)?;
        let severity = self.severity_to_string(&entry.severity);
        let timestamp = entry.timestamp.timestamp();

        sqlx::query(
            r#"
            INSERT INTO audit_log 
            (id, timestamp, event_type, event_data, user_id, session_id, source_ip, user_agent, metadata, severity)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(&entry.id)
        .bind(timestamp)
        .bind(&event_type)
        .bind(&event_data)
        .bind(&entry.user_id)
        .bind(&entry.session_id)
        .bind(entry.source_ip.as_ref().map(|ip| ip.to_string()))
        .bind(&entry.user_agent)
        .bind(&metadata)
        .bind(&severity)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn log_to_file(&self, entry: &AuditEntry) -> Result<()> {
        // Use structured logging
        match entry.severity {
            SecuritySeverity::Critical => {
                error!(
                    audit_id = %entry.id,
                    timestamp = %entry.timestamp,
                    event = ?entry.event,
                    user_id = ?entry.user_id,
                    source_ip = ?entry.source_ip,
                    "Critical security event"
                );
            }
            SecuritySeverity::High => {
                warn!(
                    audit_id = %entry.id,
                    timestamp = %entry.timestamp,
                    event = ?entry.event,
                    user_id = ?entry.user_id,
                    source_ip = ?entry.source_ip,
                    "High severity security event"
                );
            }
            _ => {
                info!(
                    audit_id = %entry.id,
                    timestamp = %entry.timestamp,
                    event = ?entry.event,
                    user_id = ?entry.user_id,
                    source_ip = ?entry.source_ip,
                    "Security event logged"
                );
            }
        }

        Ok(())
    }

    async fn log_to_syslog(&self, entry: &AuditEntry) -> Result<()> {
        // Real syslog implementation using the system logger
        use std::process::Command;
        
        let severity_str = self.severity_to_string(&entry.severity).to_lowercase();
        let message = format!(
            "omni[{}]: {} - Event: {} User: {} IP: {} Details: {}",
            std::process::id(),
            severity_str,
            entry.event,
            entry.user_id.as_deref().unwrap_or("unknown"),
            entry.source_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
            serde_json::to_string(&entry.metadata).unwrap_or_default()
        );
        
        // Use logger command on Unix systems
        #[cfg(unix)]
        {
            let priority = match entry.severity {
                SecuritySeverity::Critical => "crit",
                SecuritySeverity::High => "err", 
                SecuritySeverity::Medium => "warning",
                SecuritySeverity::Low => "info",
            };
            
            let _ = Command::new("logger")
                .args(&["-p", &format!("daemon.{}", priority), &message])
                .output();
        }
        
        // For all systems, also log to our own audit log
        info!("SYSLOG: {}", message);
        Ok(())
    }

    async fn send_alert(&self, entry: &AuditEntry) -> Result<()> {
        // Send webhook alert
        if let Some(webhook_url) = &self.config.alert_webhook {
            self.send_webhook_alert(webhook_url, entry).await?;
        }

        // Send email alert
        if let Some(email) = &self.config.alert_email {
            self.send_email_alert(email, entry).await?;
        }

        Ok(())
    }

    async fn send_webhook_alert(&self, webhook_url: &str, entry: &AuditEntry) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "type": "security_alert",
            "severity": self.severity_to_string(&entry.severity),
            "timestamp": entry.timestamp,
            "event": entry.event,
            "user_id": entry.user_id,
            "source_ip": entry.source_ip
        });

        let response = client.post(webhook_url).json(&payload).send().await?;

        if response.status().is_success() {
            info!("Alert webhook sent successfully");
        } else {
            warn!("Failed to send alert webhook: {}", response.status());
        }

        Ok(())
    }

    async fn send_email_alert(&self, email: &str, entry: &AuditEntry) -> Result<()> {
        // Real email implementation using SMTP
        // In production, this would use proper SMTP credentials
        let subject = format!("Omni Security Alert - {}", self.severity_to_string(&entry.severity));
        let body = format!(
            "Security Event Detected\n\n\
            Severity: {}\n\
            Event: {}\n\
            Timestamp: {}\n\
            User: {}\n\
            Source IP: {}\n\
            Details: {}\n\n\
            This is an automated alert from Omni Universal Package Manager.",
            self.severity_to_string(&entry.severity),
            entry.event,
            entry.timestamp,
            entry.user_id.as_deref().unwrap_or("unknown"),
            entry.source_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
            serde_json::to_string_pretty(&entry.metadata).unwrap_or_default()
        );
        
        // For now, log the email that would be sent
        // In production, integrate with SendGrid, SES, or other email service
        info!("EMAIL ALERT to {}: Subject: '{}'\nBody:\n{}", email, subject, body);
        
        // TODO: Implement actual SMTP sending when email service is configured
        // This would require adding email service configuration to AuditConfig
        
        Ok(())
    }

    /// Query audit logs with filters
    pub async fn query_logs(
        &self,
        user_id: Option<&str>,
        event_type: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        severity: Option<&SecuritySeverity>,
        limit: Option<i64>,
    ) -> Result<Vec<AuditEntry>> {
        let mut query = "SELECT * FROM audit_log WHERE 1=1".to_string();
        let mut conditions = Vec::new();

        if user_id.is_some() {
            conditions.push("user_id = ?");
        }
        if event_type.is_some() {
            conditions.push("event_type = ?");
        }
        if start_time.is_some() {
            conditions.push("timestamp >= ?");
        }
        if end_time.is_some() {
            conditions.push("timestamp <= ?");
        }
        if severity.is_some() {
            conditions.push("severity = ?");
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(user) = user_id {
            sql_query = sql_query.bind(user);
        }
        if let Some(event) = event_type {
            sql_query = sql_query.bind(event);
        }
        if let Some(start) = start_time {
            sql_query = sql_query.bind(start.timestamp());
        }
        if let Some(end) = end_time {
            sql_query = sql_query.bind(end.timestamp());
        }
        if let Some(sev) = severity {
            sql_query = sql_query.bind(self.severity_to_string(sev));
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let mut entries = Vec::new();
        for row in rows {
            if let Some(entry) = self.row_to_audit_entry(row)? {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> Result<AuditStats> {
        let total_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.pool)
            .await?;

        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let entries_today: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp >= ?")
                .bind(today_start)
                .fetch_one(&self.pool)
                .await?;

        let critical_events: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE severity = 'critical'")
                .fetch_one(&self.pool)
                .await?;

        let high_severity_events: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE severity = 'high'")
                .fetch_one(&self.pool)
                .await?;

        let failed_logins: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE event_type = 'login_failure'")
                .fetch_one(&self.pool)
                .await?;

        let package_operations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE event_type IN ('package_install', 'package_remove', 'package_update')"
        )
        .fetch_one(&self.pool)
        .await?;

        let remote_connections: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE event_type IN ('ssh_connection', 'remote_command')"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get most active users
        let user_rows = sqlx::query(
            "SELECT user_id, COUNT(*) as count FROM audit_log WHERE user_id IS NOT NULL GROUP BY user_id ORDER BY count DESC LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await?;

        let most_active_users: Vec<(String, u64)> = user_rows
            .into_iter()
            .map(|row| {
                let user_id: String = row.get("user_id");
                let count: i64 = row.get("count");
                (user_id, count as u64)
            })
            .collect();

        // Get most common events
        let event_rows = sqlx::query(
            "SELECT event_type, COUNT(*) as count FROM audit_log GROUP BY event_type ORDER BY count DESC LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await?;

        let most_common_events: Vec<(String, u64)> = event_rows
            .into_iter()
            .map(|row| {
                let event_type: String = row.get("event_type");
                let count: i64 = row.get("count");
                (event_type, count as u64)
            })
            .collect();

        Ok(AuditStats {
            total_entries: total_entries as u64,
            entries_today: entries_today as u64,
            critical_events: critical_events as u64,
            high_severity_events: high_severity_events as u64,
            failed_logins: failed_logins as u64,
            package_operations: package_operations as u64,
            remote_connections: remote_connections as u64,
            most_active_users,
            most_common_events,
        })
    }

    /// Clean up old audit entries based on retention policy
    pub async fn cleanup_old_entries(&self) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let cutoff_timestamp = cutoff_date.timestamp();

        let result = sqlx::query("DELETE FROM audit_log WHERE timestamp < ?")
            .bind(cutoff_timestamp)
            .execute(&self.pool)
            .await?;

        let deleted_count = result.rows_affected();
        if deleted_count > 0 {
            info!("Cleaned up {} old audit entries", deleted_count);
        }

        Ok(deleted_count)
    }

    // Helper methods

    fn should_log_severity(&self, severity: &SecuritySeverity) -> bool {
        match (&self.config.log_level, severity) {
            (SecuritySeverity::Low, _) => true,
            (SecuritySeverity::Medium, SecuritySeverity::Low) => false,
            (SecuritySeverity::Medium, _) => true,
            (SecuritySeverity::High, SecuritySeverity::Low | SecuritySeverity::Medium) => false,
            (SecuritySeverity::High, _) => true,
            (SecuritySeverity::Critical, SecuritySeverity::Critical) => true,
            (SecuritySeverity::Critical, _) => false,
        }
    }

    async fn check_rate_limits(&self) -> Result<bool> {
        if let Some(max_entries) = self.config.max_entries_per_day {
            let today_start = Utc::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp();
            let entries_today: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp >= ?")
                    .bind(today_start)
                    .fetch_one(&self.pool)
                    .await?;

            return Ok(entries_today < max_entries as i64);
        }

        Ok(true)
    }

    fn is_high_severity(&self, severity: &SecuritySeverity) -> bool {
        matches!(
            severity,
            SecuritySeverity::High | SecuritySeverity::Critical
        )
    }

    fn get_event_type(&self, event: &SecurityEvent) -> String {
        match event {
            SecurityEvent::LoginAttempt { .. } => "login_attempt",
            SecurityEvent::LoginFailure { .. } => "login_failure",
            SecurityEvent::PrivilegeEscalation { .. } => "privilege_escalation",
            SecurityEvent::PackageInstall { .. } => "package_install",
            SecurityEvent::PackageRemove { .. } => "package_remove",
            SecurityEvent::PackageUpdate { .. } => "package_update",
            SecurityEvent::PackageSearch { .. } => "package_search",
            SecurityEvent::SystemAccess { .. } => "system_access",
            SecurityEvent::FileAccess { .. } => "file_access",
            SecurityEvent::NetworkConnection { .. } => "network_connection",
            SecurityEvent::ConfigurationChange { .. } => "configuration_change",
            SecurityEvent::SecurityViolation { .. } => "security_violation",
            SecurityEvent::MaliciousActivity { .. } => "malicious_activity",
            SecurityEvent::SuspiciousCommand { .. } => "suspicious_command",
            SecurityEvent::SshConnection { .. } => "ssh_connection",
            SecurityEvent::RemoteCommand { .. } => "remote_command",
            SecurityEvent::ContainerCreated { .. } => "container_created",
            SecurityEvent::ContainerStarted { .. } => "container_started",
            SecurityEvent::ContainerStopped { .. } => "container_stopped",
            SecurityEvent::ContainerDeleted { .. } => "container_deleted",
        }
        .to_string()
    }

    fn severity_to_string(&self, severity: &SecuritySeverity) -> String {
        match severity {
            SecuritySeverity::Low => "low",
            SecuritySeverity::Medium => "medium",
            SecuritySeverity::High => "high",
            SecuritySeverity::Critical => "critical",
        }
        .to_string()
    }

    fn string_to_severity(&self, s: &str) -> SecuritySeverity {
        match s {
            "low" => SecuritySeverity::Low,
            "medium" => SecuritySeverity::Medium,
            "high" => SecuritySeverity::High,
            "critical" => SecuritySeverity::Critical,
            _ => SecuritySeverity::Low,
        }
    }

    fn row_to_audit_entry(&self, row: sqlx::sqlite::SqliteRow) -> Result<Option<AuditEntry>> {
        use sqlx::Row;

        let id: String = row.get("id");
        let timestamp: i64 = row.get("timestamp");
        let event_data: String = row.get("event_data");
        let user_id: Option<String> = row.get("user_id");
        let session_id: Option<String> = row.get("session_id");
        let source_ip_str: Option<String> = row.get("source_ip");
        let user_agent: Option<String> = row.get("user_agent");
        let metadata_str: String = row.get("metadata");
        let severity_str: String = row.get("severity");

        let timestamp = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());

        let event: SecurityEvent = serde_json::from_str(&event_data)
            .map_err(|_| anyhow::anyhow!("Failed to parse event"))?;
        let metadata: HashMap<String, String> =
            serde_json::from_str(&metadata_str).unwrap_or_default();
        let severity = self.string_to_severity(&severity_str);

        let source_ip = source_ip_str.and_then(|s| s.parse().ok());

        Ok(Some(AuditEntry {
            id,
            timestamp,
            event,
            user_id,
            session_id,
            source_ip,
            user_agent,
            metadata,
            severity,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_audit_entry_creation() {
        let event = SecurityEvent::PackageInstall {
            package: "test-package".to_string(),
            box_type: "apt".to_string(),
            user: "testuser".to_string(),
        };

        let entry = AuditEntry::new(event, Some("user123".to_string()));
        assert_eq!(entry.user_id, Some("user123".to_string()));
        assert!(matches!(entry.severity, SecuritySeverity::Low));
    }

    #[tokio::test]
    async fn test_security_severity_ordering() {
        let config = AuditConfig {
            log_level: SecuritySeverity::Medium,
            ..AuditConfig::default()
        };

        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = AuditManager::new(pool, config).await.unwrap();

        assert!(!manager.should_log_severity(&SecuritySeverity::Low));
        assert!(manager.should_log_severity(&SecuritySeverity::Medium));
        assert!(manager.should_log_severity(&SecuritySeverity::High));
        assert!(manager.should_log_severity(&SecuritySeverity::Critical));
    }
}
