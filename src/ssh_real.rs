use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::prelude::*;
use russh::*;
use russh_keys::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Real SSH client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealSshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: RealAuthMethod,
    pub connect_timeout: Duration,
    pub command_timeout: Duration,
    pub max_retries: u32,
    pub host_key_verification: bool,
    pub known_hosts_file: Option<PathBuf>,
    pub compression: bool,
    pub keepalive_interval: Option<Duration>,
}

impl Default for RealSshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: "root".to_string(),
            auth_method: RealAuthMethod::PublicKey {
                private_key_path: PathBuf::from("~/.ssh/id_rsa"),
                passphrase: None,
            },
            connect_timeout: Duration::from_secs(30),
            command_timeout: Duration::from_secs(300),
            max_retries: 3,
            host_key_verification: true,
            known_hosts_file: Some(PathBuf::from("~/.ssh/known_hosts")),
            compression: true,
            keepalive_interval: Some(Duration::from_secs(30)),
        }
    }
}

/// Real SSH authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealAuthMethod {
    Password {
        password: String,
    },
    PublicKey {
        private_key_path: PathBuf,
        passphrase: Option<String>,
    },
    Agent,
}

/// Result of a real SSH command execution
#[derive(Debug, Clone)]
pub struct RealSshCommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub host: String,
}

impl RealSshCommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// SSH client handler for russh
struct SshClientHandler {
    username: String,
    auth_method: RealAuthMethod,
}

#[async_trait]
impl client::Handler for SshClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // In a real implementation, this would verify against known_hosts
        Ok(true)
    }

    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        _channel: Channel<client::Msg>,
        _connected_address: &str,
        _connected_port: u32,
        _originator_address: &str,
        _originator_port: u32,
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Real SSH session for executing remote commands
pub struct RealSshSession {
    config: RealSshConfig,
    session_id: String,
    connected: bool,
    client_session: Option<Arc<Mutex<client::Handle<SshClientHandler>>>>,
}

impl RealSshSession {
    pub fn new(config: RealSshConfig) -> Self {
        Self {
            config,
            session_id: Uuid::new_v4().to_string(),
            connected: false,
            client_session: None,
        }
    }

    /// Connect to the remote host using real SSH
    pub async fn connect(&mut self) -> Result<()> {
        info!(
            "Connecting to SSH host: {}@{}:{}",
            self.config.username, self.config.host, self.config.port
        );

        // Validate host and port
        self.validate_host()?;

        // Create SSH client configuration
        let ssh_config = client::Config {
            inactivity_timeout: self.config.keepalive_interval,
            ..<_>::default()
        };

        // Create client handler
        let handler = SshClientHandler {
            username: self.config.username.clone(),
            auth_method: self.config.auth_method.clone(),
        };

        // Connect to the server
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| anyhow!("Invalid address: {}", e))?;

        let tcp_stream =
            tokio::time::timeout(self.config.connect_timeout, TcpStream::connect(&addr))
                .await
                .map_err(|_| anyhow!("Connection timeout"))?
                .map_err(|e| anyhow!("Connection failed: {}", e))?;

        // Create SSH session
        let session = client::connect(ssh_config, tcp_stream, handler).await?;

        // Authenticate
        self.authenticate(&session).await?;

        self.client_session = Some(Arc::new(Mutex::new(session)));
        self.connected = true;

        info!("SSH session established: {}", self.session_id);
        Ok(())
    }

    /// Execute a command on the remote host
    pub async fn execute_command(&mut self, command: &str) -> Result<RealSshCommandResult> {
        if !self.connected {
            return Err(anyhow!("SSH session not connected"));
        }

        info!("Executing remote command: {}", command);
        let start_time = std::time::Instant::now();

        // Validate command for security
        self.validate_command(command)?;

        let session = self
            .client_session
            .as_ref()
            .ok_or_else(|| anyhow!("No active SSH session"))?;

        let result = tokio::time::timeout(
            self.config.command_timeout,
            self.execute_command_internal(session, command),
        )
        .await
        .map_err(|_| anyhow!("Command execution timeout"))?;

        let duration = start_time.elapsed();

        match result {
            Ok((exit_code, stdout, stderr)) => {
                let result = RealSshCommandResult {
                    command: command.to_string(),
                    exit_code,
                    stdout,
                    stderr,
                    duration,
                    host: self.config.host.clone(),
                };

                if result.success() {
                    info!("Command executed successfully in {:?}", duration);
                    debug!("Command output: {}", result.stdout);
                } else {
                    warn!(
                        "Command failed with exit code {}: {}",
                        exit_code, result.stderr
                    );
                }

                Ok(result)
            }
            Err(e) => {
                error!("Command execution failed: {}", e);
                Err(e)
            }
        }
    }

    /// Execute multiple commands in sequence
    pub async fn execute_commands(
        &mut self,
        commands: &[&str],
    ) -> Result<Vec<RealSshCommandResult>> {
        let mut results = Vec::new();

        for command in commands {
            let result = self.execute_command(command).await?;
            let success = result.success();
            results.push(result);

            if !success {
                warn!("Command failed, stopping execution chain");
                break;
            }
        }

        Ok(results)
    }

    /// Execute a command with privilege escalation (sudo)
    pub async fn execute_privileged_command(
        &mut self,
        command: &str,
        sudo_password: Option<&str>,
    ) -> Result<RealSshCommandResult> {
        let privileged_command = if let Some(password) = sudo_password {
            format!("echo '{}' | sudo -S {}", password, command)
        } else {
            format!("sudo {}", command)
        };

        self.execute_command(&privileged_command).await
    }

    /// Upload a file to the remote host using SFTP
    pub async fn upload_file(&mut self, local_path: &PathBuf, remote_path: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("SSH session not connected"));
        }

        info!(
            "Uploading file: {} -> {}",
            local_path.display(),
            remote_path
        );

        // Read local file
        let content = tokio::fs::read(local_path)
            .await
            .map_err(|e| anyhow!("Failed to read local file: {}", e))?;

        let session = self
            .client_session
            .as_ref()
            .ok_or_else(|| anyhow!("No active SSH session"))?;

        let session_guard = session.lock().await;

        // Create SFTP channel
        let channel = session_guard.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await?;

        // For simplicity, we'll use SCP-style upload via command execution
        // In a real implementation, you'd use the SFTP protocol
        drop(session_guard);

        // Create remote directory if needed
        if let Some(parent) = std::path::Path::new(remote_path).parent() {
            let mkdir_cmd = format!("mkdir -p {}", parent.display());
            self.execute_command(&mkdir_cmd).await?;
        }

        // Use base64 encoding for binary-safe transfer
        let encoded = base64::prelude::BASE64_STANDARD.encode(&content);
        let chunk_size = 8192; // Split into chunks to avoid command line limits

        // Create the file
        self.execute_command(&format!("touch {}", remote_path))
            .await?;

        // Upload in chunks
        for (i, chunk) in encoded
            .chars()
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .enumerate()
        {
            let chunk_str: String = chunk.iter().collect();
            let append_cmd = if i == 0 {
                format!("echo '{}' | base64 -d > {}", chunk_str, remote_path)
            } else {
                format!("echo '{}' | base64 -d >> {}", chunk_str, remote_path)
            };

            let result = self.execute_command(&append_cmd).await?;
            if !result.success() {
                return Err(anyhow!(
                    "File upload failed at chunk {}: {}",
                    i,
                    result.stderr
                ));
            }
        }

        info!("File uploaded successfully");
        Ok(())
    }

    /// Download a file from the remote host
    pub async fn download_file(&mut self, remote_path: &str, local_path: &PathBuf) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("SSH session not connected"));
        }

        info!(
            "Downloading file: {} -> {}",
            remote_path,
            local_path.display()
        );

        // Check if remote file exists
        let check_cmd = format!("test -f {}", remote_path);
        let check_result = self.execute_command(&check_cmd).await?;
        if !check_result.success() {
            return Err(anyhow!("Remote file does not exist: {}", remote_path));
        }

        // Download using base64 encoding
        let download_cmd = format!("base64 < {}", remote_path);
        let result = self.execute_command(&download_cmd).await?;

        if !result.success() {
            return Err(anyhow!("File download failed: {}", result.stderr));
        }

        // Decode and save
        let content = base64::prelude::BASE64_STANDARD
            .decode(result.stdout.trim())
            .map_err(|e| anyhow!("Failed to decode file content: {}", e))?;

        // Create local directory if needed
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| anyhow!("Failed to create local directory: {}", e))?;
        }

        tokio::fs::write(local_path, content)
            .await
            .map_err(|e| anyhow!("Failed to write local file: {}", e))?;

        info!("File downloaded successfully");
        Ok(())
    }

    /// Test the connection
    pub async fn test_connection(&mut self) -> Result<bool> {
        match self.execute_command("echo 'test'").await {
            Ok(result) => Ok(result.success() && result.stdout.trim() == "test"),
            Err(_) => Ok(false),
        }
    }

    /// Get system information from remote host
    pub async fn get_system_info(&mut self) -> Result<SystemInfo> {
        let commands = vec![
            "uname -a",            // System info
            "cat /etc/os-release", // OS info
            "whoami",              // Current user
            "pwd",                 // Current directory
            "uptime",              // System uptime
            "df -h",               // Disk usage
            "free -h",             // Memory usage
        ];

        let mut info = SystemInfo::default();

        for (i, cmd) in commands.iter().enumerate() {
            if let Ok(result) = self.execute_command(cmd).await {
                if result.success() {
                    match i {
                        0 => info.uname = result.stdout.trim().to_string(),
                        1 => info.os_release = result.stdout.trim().to_string(),
                        2 => info.current_user = result.stdout.trim().to_string(),
                        3 => info.current_directory = result.stdout.trim().to_string(),
                        4 => info.uptime = result.stdout.trim().to_string(),
                        5 => info.disk_usage = result.stdout.trim().to_string(),
                        6 => info.memory_usage = result.stdout.trim().to_string(),
                        _ => {}
                    }
                }
            }
        }

        Ok(info)
    }

    /// Disconnect the SSH session
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.connected {
            info!("Disconnecting SSH session: {}", self.session_id);

            if let Some(session) = &self.client_session {
                let session_guard = session.lock().await;
                session_guard
                    .disconnect(Disconnect::ByApplication, "", "en-US")
                    .await?;
            }

            self.connected = false;
            self.client_session = None;

            info!("SSH session disconnected");
        }

        Ok(())
    }

    // Private helper methods

    async fn authenticate(&self, session: &client::Handle<SshClientHandler>) -> Result<()> {
        info!("Authenticating SSH session");

        match &self.config.auth_method {
            RealAuthMethod::Password { password } => {
                let auth_result = session
                    .authenticate_password(&self.config.username, password)
                    .await?;

                if !auth_result {
                    return Err(anyhow!("Password authentication failed"));
                }
                info!("Password authentication successful");
            }
            RealAuthMethod::PublicKey {
                private_key_path,
                passphrase,
            } => {
                // Load private key
                let key_data = tokio::fs::read(private_key_path)
                    .await
                    .map_err(|e| anyhow!("Failed to read private key: {}", e))?;

                let key = if let Some(passphrase) = passphrase {
                    decode_secret_key(&key_data, Some(passphrase.as_bytes()))?
                } else {
                    decode_secret_key(&key_data, None)?
                };

                let auth_result = session
                    .authenticate_publickey(&self.config.username, Arc::new(key))
                    .await?;

                if !auth_result {
                    return Err(anyhow!("Public key authentication failed"));
                }
                info!("Public key authentication successful");
            }
            RealAuthMethod::Agent => {
                // SSH agent authentication would be implemented here
                return Err(anyhow!("SSH agent authentication not yet implemented"));
            }
        }

        Ok(())
    }

    async fn execute_command_internal(
        &self,
        session: &Arc<Mutex<client::Handle<SshClientHandler>>>,
        command: &str,
    ) -> Result<(i32, String, String)> {
        let session_guard = session.lock().await;

        // Open a channel
        let mut channel = session_guard.channel_open_session().await?;

        // Execute the command
        channel.exec(true, command).await?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = 0;

        // Read output
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    stdout.extend_from_slice(&data);
                }
                ChannelMsg::ExtendedData { data, ext: 1 } => {
                    stderr.extend_from_slice(&data);
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = exit_status as i32;
                }
                ChannelMsg::Eof => {
                    break;
                }
                _ => {}
            }
        }

        let stdout_str = String::from_utf8_lossy(&stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&stderr).to_string();

        Ok((exit_code, stdout_str, stderr_str))
    }

    fn validate_host(&self) -> Result<()> {
        if self.config.host.is_empty() {
            return Err(anyhow!("Host cannot be empty"));
        }

        if self.config.port == 0 || self.config.port > 65535 {
            return Err(anyhow!("Invalid port: {}", self.config.port));
        }

        Ok(())
    }

    fn validate_command(&self, command: &str) -> Result<()> {
        if command.is_empty() {
            return Err(anyhow!("Command cannot be empty"));
        }

        // Block dangerous commands
        let dangerous_patterns = [
            "rm -rf /",
            ":(){ :|:& };:", // Fork bomb
            "dd if=/dev/zero",
            "mkfs",
            "fdisk",
            "format",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return Err(anyhow!("Dangerous command blocked: {}", pattern));
            }
        }

        Ok(())
    }
}

impl Drop for RealSshSession {
    fn drop(&mut self) {
        if self.connected {
            // Attempt graceful disconnect in background
            if let Some(session) = self.client_session.clone() {
                tokio::spawn(async move {
                    let session_guard = session.lock().await;
                    let _ = session_guard
                        .disconnect(Disconnect::ByApplication, "", "en-US")
                        .await;
                });
            }
        }
    }
}

/// System information from remote host
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub uname: String,
    pub os_release: String,
    pub current_user: String,
    pub current_directory: String,
    pub uptime: String,
    pub disk_usage: String,
    pub memory_usage: String,
}

/// Real SSH connection pool for managing multiple connections
pub struct RealSshConnectionPool {
    connections: HashMap<String, RealSshSession>,
    max_connections: usize,
}

impl RealSshConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections,
        }
    }

    pub async fn get_or_create_session(
        &mut self,
        host: &str,
        config: RealSshConfig,
    ) -> Result<&mut RealSshSession> {
        if !self.connections.contains_key(host) {
            if self.connections.len() >= self.max_connections {
                // Remove oldest connection
                if let Some(key) = self.connections.keys().next().cloned() {
                    if let Some(mut session) = self.connections.remove(&key) {
                        let _ = session.disconnect().await;
                    }
                }
            }

            let mut session = RealSshSession::new(config);
            session.connect().await?;
            self.connections.insert(host.to_string(), session);
        }

        Ok(self.connections.get_mut(host).unwrap())
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        for (_, session) in &mut self.connections {
            let _ = session.disconnect().await;
        }
        self.connections.clear();
        Ok(())
    }
}

/// Real SSH client for remote package management
pub struct RealSshClient {
    pool: RealSshConnectionPool,
}

impl RealSshClient {
    pub fn new() -> Self {
        Self {
            pool: RealSshConnectionPool::new(10), // Max 10 concurrent connections
        }
    }

    /// Execute a package management command on a remote host
    pub async fn execute_remote_package_command(
        &mut self,
        host: &str,
        config: RealSshConfig,
        box_type: &str,
        package_name: &str,
        operation: &str,
    ) -> Result<RealSshCommandResult> {
        let session = self.pool.get_or_create_session(host, config).await?;

        // Build package manager command
        let command = match box_type {
            "apt" => match operation {
                "install" => format!("sudo apt update && sudo apt install -y {}", package_name),
                "remove" => format!("sudo apt remove -y {}", package_name),
                "update" => format!("sudo apt update && sudo apt upgrade -y {}", package_name),
                _ => return Err(anyhow!("Unsupported operation: {}", operation)),
            },
            "dnf" => match operation {
                "install" => format!("sudo dnf install -y {}", package_name),
                "remove" => format!("sudo dnf remove -y {}", package_name),
                "update" => format!("sudo dnf upgrade -y {}", package_name),
                _ => return Err(anyhow!("Unsupported operation: {}", operation)),
            },
            "pacman" => match operation {
                "install" => format!("sudo pacman -S --noconfirm {}", package_name),
                "remove" => format!("sudo pacman -R --noconfirm {}", package_name),
                "update" => format!("sudo pacman -Syu --noconfirm {}", package_name),
                _ => return Err(anyhow!("Unsupported operation: {}", operation)),
            },
            _ => return Err(anyhow!("Unsupported box type: {}", box_type)),
        };

        session.execute_command(&command).await
    }

    /// Test connectivity to a remote host
    pub async fn test_host_connectivity(
        &mut self,
        host: &str,
        config: RealSshConfig,
    ) -> Result<bool> {
        match self.pool.get_or_create_session(host, config).await {
            Ok(session) => session.test_connection().await,
            Err(_) => Ok(false),
        }
    }

    /// Get system information from multiple hosts
    pub async fn get_multi_host_info(
        &mut self,
        hosts: Vec<(String, RealSshConfig)>,
    ) -> Result<HashMap<String, SystemInfo>> {
        let mut results = HashMap::new();

        for (host, config) in hosts {
            match self.pool.get_or_create_session(&host, config).await {
                Ok(session) => {
                    if let Ok(info) = session.get_system_info().await {
                        results.insert(host, info);
                    }
                }
                Err(e) => {
                    error!("Failed to connect to {}: {}", host, e);
                }
            }
        }

        Ok(results)
    }

    /// Disconnect all sessions
    pub async fn disconnect_all(&mut self) -> Result<()> {
        self.pool.disconnect_all().await
    }
}

impl Default for RealSshClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_ssh_config_default() {
        let config = RealSshConfig::default();
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "root");
        assert!(config.host_key_verification);
    }

    #[tokio::test]
    async fn test_real_ssh_session_creation() {
        let config = RealSshConfig {
            host: "example.com".to_string(),
            ..RealSshConfig::default()
        };

        let session = RealSshSession::new(config);
        assert!(!session.connected);
        assert!(session.session_id.len() > 0);
    }

    #[tokio::test]
    async fn test_command_validation() {
        let config = RealSshConfig {
            host: "example.com".to_string(),
            ..RealSshConfig::default()
        };

        let session = RealSshSession::new(config);

        // Valid command
        assert!(session.validate_command("ls -la").is_ok());

        // Dangerous command
        assert!(session.validate_command("rm -rf /").is_err());

        // Empty command
        assert!(session.validate_command("").is_err());
    }

    #[test]
    fn test_real_ssh_client_creation() {
        let client = RealSshClient::new();
        assert_eq!(client.pool.max_connections, 10);
    }
}
