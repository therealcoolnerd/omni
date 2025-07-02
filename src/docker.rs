use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Docker container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image: String,
    pub tag: String,
    pub name: Option<String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub user: Option<String>,
    pub network: Option<String>,
    pub restart_policy: RestartPolicy,
    pub resources: ResourceLimits,
    pub security_options: SecurityOptions,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            tag: "latest".to_string(),
            name: None,
            ports: Vec::new(),
            volumes: Vec::new(),
            environment: HashMap::new(),
            working_directory: Some("/workdir".to_string()),
            user: None,
            network: None,
            restart_policy: RestartPolicy::No,
            resources: ResourceLimits::default(),
            security_options: SecurityOptions::default(),
        }
    }
}

/// Port mapping for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: PortProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortProtocol {
    TCP,
    UDP,
}

/// Volume mapping for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: PathBuf,
    pub container_path: String,
    pub mode: VolumeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeMode {
    ReadOnly,
    ReadWrite,
}

/// Container restart policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    No,
    Always,
    OnFailure,
    UnlessStopped,
}

/// Resource limits for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_mb: Option<u64>,
    pub cpu_cores: Option<f64>,
    pub disk_space_mb: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_mb: Some(512), // 512MB default
            cpu_cores: Some(1.0), // 1 CPU core default
            disk_space_mb: None,
        }
    }
}

/// Security options for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOptions {
    pub read_only_root: bool,
    pub no_new_privileges: bool,
    pub user_namespace: bool,
    pub capabilities_drop: Vec<String>,
    pub capabilities_add: Vec<String>,
}

impl Default for SecurityOptions {
    fn default() -> Self {
        Self {
            read_only_root: false,
            no_new_privileges: true,
            user_namespace: false,
            capabilities_drop: vec![
                "ALL".to_string(), // Drop all capabilities by default
            ],
            capabilities_add: vec![
                "CHOWN".to_string(),
                "SETUID".to_string(),
                "SETGID".to_string(),
            ],
        }
    }
}

/// Information about a Docker container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub created: String,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
}

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Paused,
    Restarting,
    Dead,
    Created,
}

/// Result of a command execution in a container
#[derive(Debug, Clone)]
pub struct DockerCommandResult {
    pub container_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl DockerCommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Docker client for container management
pub struct DockerClient {
    docker_command: String,
}

impl DockerClient {
    pub async fn new() -> Result<Self> {
        // Check if Docker is available
        let docker_command = if Self::is_command_available("docker").await? {
            "docker".to_string()
        } else if Self::is_command_available("podman").await? {
            info!("Docker not found, using Podman as alternative");
            "podman".to_string()
        } else {
            return Err(anyhow!("Neither Docker nor Podman is available"));
        };

        info!("Using container runtime: {}", docker_command);

        Ok(Self { docker_command })
    }

    /// Check if Docker daemon is running
    pub async fn check_daemon(&self) -> Result<bool> {
        let output = Command::new(&self.docker_command)
            .arg("info")
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Create and start a new container
    pub async fn create_container(&self, config: &DockerConfig) -> Result<String> {
        info!(
            "Creating container from image: {}:{}",
            config.image, config.tag
        );

        let container_name = config
            .name
            .clone()
            .unwrap_or_else(|| format!("omni-{}", Uuid::new_v4().to_string()[..8].to_string()));

        let mut args = vec![
            "run".to_string(),
            "-d".to_string(), // Detached mode
            "--name".to_string(),
            container_name.clone(),
        ];

        // Add security options
        if config.security_options.read_only_root {
            args.extend(vec!["--read-only".to_string()]);
        }

        if config.security_options.no_new_privileges {
            args.extend(vec![
                "--security-opt".to_string(),
                "no-new-privileges".to_string(),
            ]);
        }

        // Drop capabilities
        for cap in &config.security_options.capabilities_drop {
            args.extend(vec!["--cap-drop".to_string(), cap.clone()]);
        }

        // Add capabilities
        for cap in &config.security_options.capabilities_add {
            args.extend(vec!["--cap-add".to_string(), cap.clone()]);
        }

        // Add resource limits
        if let Some(memory) = config.resources.memory_mb {
            args.extend(vec!["-m".to_string(), format!("{}m", memory)]);
        }

        if let Some(cpu) = config.resources.cpu_cores {
            args.extend(vec!["--cpus".to_string(), cpu.to_string()]);
        }

        // Add port mappings
        for port in &config.ports {
            let protocol = match port.protocol {
                PortProtocol::TCP => "tcp",
                PortProtocol::UDP => "udp",
            };
            args.extend(vec![
                "-p".to_string(),
                format!("{}:{}/{}", port.host_port, port.container_port, protocol),
            ]);
        }

        // Add volume mappings
        for volume in &config.volumes {
            let mode = match volume.mode {
                VolumeMode::ReadOnly => "ro",
                VolumeMode::ReadWrite => "rw",
            };
            args.extend(vec![
                "-v".to_string(),
                format!(
                    "{}:{}:{}",
                    volume.host_path.display(),
                    volume.container_path,
                    mode
                ),
            ]);
        }

        // Add environment variables
        for (key, value) in &config.environment {
            args.extend(vec!["-e".to_string(), format!("{}={}", key, value)]);
        }

        // Add working directory
        if let Some(workdir) = &config.working_directory {
            args.extend(vec!["-w".to_string(), workdir.clone()]);
        }

        // Add user
        if let Some(user) = &config.user {
            args.extend(vec!["-u".to_string(), user.clone()]);
        }

        // Add network
        if let Some(network) = &config.network {
            args.extend(vec!["--network".to_string(), network.clone()]);
        }

        // Add restart policy
        let restart_policy = match config.restart_policy {
            RestartPolicy::No => "no",
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::UnlessStopped => "unless-stopped",
        };
        args.extend(vec!["--restart".to_string(), restart_policy.to_string()]);

        // Add image
        args.push(format!("{}:{}", config.image, config.tag));

        // Execute docker run command
        let output = Command::new(&self.docker_command)
            .args(&args)
            .output()
            .await?;

        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!(
                "Container created successfully: {} ({})",
                container_name, container_id
            );
            Ok(container_id)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("Failed to create container: {}", error);
            Err(anyhow!("Container creation failed: {}", error))
        }
    }

    /// Execute a command in a running container
    pub async fn execute_command(
        &self,
        container_id: &str,
        command: &str,
        user: Option<&str>,
    ) -> Result<DockerCommandResult> {
        info!(
            "Executing command in container {}: {}",
            container_id, command
        );
        let start_time = std::time::Instant::now();

        let mut args = vec!["exec".to_string()];

        // Add user if specified
        if let Some(user) = user {
            args.extend(vec!["-u".to_string(), user.to_string()]);
        }

        args.extend(vec![
            container_id.to_string(),
            "sh".to_string(),
            "-c".to_string(),
            command.to_string(),
        ]);

        let output = Command::new(&self.docker_command)
            .args(&args)
            .output()
            .await?;

        let duration = start_time.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);

        let result = DockerCommandResult {
            container_id: container_id.to_string(),
            command: command.to_string(),
            exit_code,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
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

    /// Install a package in a container
    pub async fn install_package_in_container(
        &self,
        container_id: &str,
        package_manager: &str,
        package_name: &str,
    ) -> Result<DockerCommandResult> {
        let command = match package_manager {
            "apt" => format!("apt update && apt install -y {}", package_name),
            "dnf" => format!("dnf install -y {}", package_name),
            "pacman" => format!("pacman -S --noconfirm {}", package_name),
            "zypper" => format!("zypper install -y {}", package_name),
            "emerge" => format!("emerge {}", package_name),
            "apk" => format!("apk add {}", package_name), // Alpine Linux
            _ => return Err(anyhow!("Unsupported package manager: {}", package_manager)),
        };

        self.execute_command(container_id, &command, Some("root"))
            .await
    }

    /// Stop a container
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        info!("Stopping container: {}", container_id);

        let output = Command::new(&self.docker_command)
            .args(&["stop", container_id])
            .output()
            .await?;

        if output.status.success() {
            info!("Container stopped successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to stop container: {}", error))
        }
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str, force: bool) -> Result<()> {
        info!("Removing container: {}", container_id);

        let mut args = vec!["rm".to_string()];
        if force {
            args.push("-f".to_string());
        }
        args.push(container_id.to_string());

        let output = Command::new(&self.docker_command)
            .args(&args)
            .output()
            .await?;

        if output.status.success() {
            info!("Container removed successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to remove container: {}", error))
        }
    }

    /// List all containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let mut args = vec!["ps".to_string(), "--format".to_string(), "json".to_string()];
        if all {
            args.push("-a".to_string());
        }

        let output = Command::new(&self.docker_command)
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to list containers: {}", error));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();

        // Parse JSON output line by line
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    if let Some(container) = self.parse_container_json(&json) {
                        containers.push(container);
                    }
                }
                Err(e) => {
                    warn!("Failed to parse container JSON: {}", e);
                }
            }
        }

        Ok(containers)
    }

    /// Get container information
    pub async fn get_container_info(&self, container_id: &str) -> Result<ContainerInfo> {
        let output = Command::new(&self.docker_command)
            .args(&["inspect", container_id])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to inspect container: {}", error));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)?;

        if let Some(container_array) = json.as_array() {
            if let Some(container_json) = container_array.first() {
                if let Some(info) = self.parse_container_inspect_json(container_json) {
                    return Ok(info);
                }
            }
        }

        Err(anyhow!("Failed to parse container information"))
    }

    /// Copy file to container
    pub async fn copy_to_container(
        &self,
        container_id: &str,
        local_path: &PathBuf,
        container_path: &str,
    ) -> Result<()> {
        info!(
            "Copying file to container: {} -> {}",
            local_path.display(),
            container_path
        );

        let output = Command::new(&self.docker_command)
            .args(&[
                "cp",
                &local_path.to_string_lossy(),
                &format!("{}:{}", container_id, container_path),
            ])
            .output()
            .await?;

        if output.status.success() {
            info!("File copied successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to copy file: {}", error))
        }
    }

    /// Copy file from container
    pub async fn copy_from_container(
        &self,
        container_id: &str,
        container_path: &str,
        local_path: &PathBuf,
    ) -> Result<()> {
        info!(
            "Copying file from container: {} -> {}",
            container_path,
            local_path.display()
        );

        let output = Command::new(&self.docker_command)
            .args(&[
                "cp",
                &format!("{}:{}", container_id, container_path),
                &local_path.to_string_lossy(),
            ])
            .output()
            .await?;

        if output.status.success() {
            info!("File copied successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to copy file: {}", error))
        }
    }

    /// Pull an image
    pub async fn pull_image(&self, image: &str, tag: &str) -> Result<()> {
        info!("Pulling image: {}:{}", image, tag);

        let output = Command::new(&self.docker_command)
            .args(&["pull", &format!("{}:{}", image, tag)])
            .output()
            .await?;

        if output.status.success() {
            info!("Image pulled successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to pull image: {}", error))
        }
    }

    /// Clean up unused containers and images
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up Docker resources");

        // Remove stopped containers
        let _ = Command::new(&self.docker_command)
            .args(&["container", "prune", "-f"])
            .output()
            .await;

        // Remove unused images
        let _ = Command::new(&self.docker_command)
            .args(&["image", "prune", "-f"])
            .output()
            .await;

        // Remove unused volumes
        let _ = Command::new(&self.docker_command)
            .args(&["volume", "prune", "-f"])
            .output()
            .await;

        info!("Docker cleanup completed");
        Ok(())
    }

    // Private helper methods

    async fn is_command_available(command: &str) -> Result<bool> {
        let output = Command::new("which").arg(command).output().await?;

        Ok(output.status.success())
    }

    fn parse_container_json(&self, json: &serde_json::Value) -> Option<ContainerInfo> {
        let id = json.get("ID")?.as_str()?.to_string();
        let name = json.get("Names")?.as_str()?.to_string();
        let image = json.get("Image")?.as_str()?.to_string();
        let created = json.get("CreatedAt")?.as_str()?.to_string();
        let status_str = json.get("Status")?.as_str()?;

        let status = if status_str.contains("Up") {
            ContainerStatus::Running
        } else if status_str.contains("Exited") {
            ContainerStatus::Stopped
        } else {
            ContainerStatus::Created
        };

        Some(ContainerInfo {
            id,
            name,
            image,
            status,
            created,
            ports: Vec::new(),   // Simplified for now
            volumes: Vec::new(), // Simplified for now
        })
    }

    fn parse_container_inspect_json(&self, json: &serde_json::Value) -> Option<ContainerInfo> {
        let id = json.get("Id")?.as_str()?.to_string();
        let name = json
            .get("Name")?
            .as_str()?
            .trim_start_matches('/')
            .to_string();
        let image = json.get("Config")?.get("Image")?.as_str()?.to_string();
        let created = json.get("Created")?.as_str()?.to_string();

        let state = json.get("State")?;
        let status = if state.get("Running")?.as_bool()? {
            ContainerStatus::Running
        } else if state.get("Paused")?.as_bool().unwrap_or(false) {
            ContainerStatus::Paused
        } else if state.get("Restarting")?.as_bool().unwrap_or(false) {
            ContainerStatus::Restarting
        } else if state.get("Dead")?.as_bool().unwrap_or(false) {
            ContainerStatus::Dead
        } else {
            ContainerStatus::Stopped
        };

        Some(ContainerInfo {
            id,
            name,
            image,
            status,
            created,
            ports: Vec::new(),   // Could parse from NetworkSettings.Ports
            volumes: Vec::new(), // Could parse from Mounts
        })
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { Self::new().await.expect("Failed to create Docker client") })
    }
}

/// Docker package manager for containerized package management
pub struct DockerPackageManager {
    client: DockerClient,
    base_images: HashMap<String, String>,
}

impl DockerPackageManager {
    pub async fn new() -> Result<Self> {
        let mut base_images = HashMap::new();
        base_images.insert("apt".to_string(), "ubuntu:22.04".to_string());
        base_images.insert("dnf".to_string(), "fedora:39".to_string());
        base_images.insert("pacman".to_string(), "archlinux:latest".to_string());
        base_images.insert("zypper".to_string(), "opensuse/leap:15.5".to_string());
        base_images.insert("emerge".to_string(), "gentoo/stage3".to_string());
        base_images.insert("apk".to_string(), "alpine:latest".to_string());

        Ok(Self {
            client: DockerClient::new().await?,
            base_images,
        })
    }

    /// Install a package in an isolated container
    pub async fn install_package_isolated(
        &self,
        package_manager: &str,
        package_name: &str,
    ) -> Result<DockerCommandResult> {
        // Get base image for package manager
        let image = self
            .base_images
            .get(package_manager)
            .ok_or_else(|| anyhow!("Unsupported package manager: {}", package_manager))?;

        // Create container configuration
        let config = DockerConfig {
            image: image.split(':').next().unwrap().to_string(),
            tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
            name: Some(format!("omni-{}-{}", package_manager, package_name)),
            security_options: SecurityOptions {
                read_only_root: false, // Allow package installation
                ..SecurityOptions::default()
            },
            ..DockerConfig::default()
        };

        // Pull image first
        self.client.pull_image(&config.image, &config.tag).await?;

        // Create container
        let container_id = self.client.create_container(&config).await?;

        // Install package
        let result = self
            .client
            .install_package_in_container(&container_id, package_manager, package_name)
            .await;

        // Clean up container
        let _ = self.client.stop_container(&container_id).await;
        let _ = self.client.remove_container(&container_id, true).await;

        result
    }

    /// Test package installation in multiple distros
    pub async fn test_package_compatibility(
        &self,
        package_name: &str,
    ) -> Result<HashMap<String, DockerCommandResult>> {
        let mut results = HashMap::new();

        for (pm, _) in &self.base_images {
            info!("Testing {} installation with {}", package_name, pm);

            match self.install_package_isolated(pm, package_name).await {
                Ok(result) => {
                    results.insert(pm.clone(), result);
                }
                Err(e) => {
                    error!("Failed to test with {}: {}", pm, e);
                    // Create a failed result
                    results.insert(
                        pm.clone(),
                        DockerCommandResult {
                            container_id: "failed".to_string(),
                            command: format!("install {}", package_name),
                            exit_code: 1,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            duration: Duration::from_secs(0),
                        },
                    );
                }
            }
        }

        Ok(results)
    }
}

impl Default for DockerPackageManager {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::new())
            .expect("Failed to create Docker package manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_config_default() {
        let config = DockerConfig::default();
        assert_eq!(config.tag, "latest");
        assert!(config.security_options.no_new_privileges);
        assert_eq!(config.resources.memory_mb, Some(512));
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.memory_mb, Some(512));
        assert_eq!(limits.cpu_cores, Some(1.0));
    }

    #[test]
    fn test_security_options_default() {
        let security = SecurityOptions::default();
        assert!(security.no_new_privileges);
        assert!(!security.read_only_root);
        assert!(security.capabilities_drop.contains(&"ALL".to_string()));
    }

    #[tokio::test]
    async fn test_docker_package_manager_creation() {
        // This test might fail if Docker is not available
        if let Ok(manager) = DockerPackageManager::new().await {
            assert!(manager.base_images.contains_key("apt"));
            assert!(manager.base_images.contains_key("dnf"));
        }
    }
}
