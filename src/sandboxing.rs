use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tracing::{info, warn, error};

/// Sandboxing utilities for isolating package operations
pub struct Sandbox {
    work_dir: TempDir,
    allowed_paths: Vec<PathBuf>,
    network_allowed: bool,
}

impl Sandbox {
    /// Create a new sandbox environment
    pub fn new() -> Result<Self> {
        let work_dir = TempDir::new()
            .map_err(|e| anyhow!("Failed to create sandbox directory: {}", e))?;
        
        info!("Created sandbox at: {:?}", work_dir.path());
        
        Ok(Self {
            work_dir,
            allowed_paths: vec![],
            network_allowed: false,
        })
    }
    
    /// Allow access to specific paths
    pub fn allow_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()
            .map_err(|e| anyhow!("Cannot canonicalize path {:?}: {}", path.as_ref(), e))?;
        
        // Validate path is safe
        self.validate_path(&path)?;
        
        self.allowed_paths.push(path.clone());
        info!("Added allowed path to sandbox: {:?}", path);
        Ok(())
    }
    
    /// Enable or disable network access
    pub fn set_network_access(&mut self, allowed: bool) {
        self.network_allowed = allowed;
        info!("Network access in sandbox: {}", if allowed { "enabled" } else { "disabled" });
    }
    
    /// Get the sandbox working directory
    pub fn work_dir(&self) -> &Path {
        self.work_dir.path()
    }
    
    /// Execute a command within the sandbox
    pub fn execute(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        info!("Executing in sandbox: {} {:?}", command, args);
        
        // Validate command is safe
        self.validate_command(command)?;
        
        let mut cmd = self.create_sandboxed_command(command, args)?;
        
        let output = cmd.output()
            .map_err(|e| anyhow!("Failed to execute sandboxed command: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Sandboxed command failed: {}", stderr);
        }
        
        Ok(output)
    }
    
    /// Create a file within the sandbox
    pub fn create_file<P: AsRef<Path>>(&self, rel_path: P, content: &[u8]) -> Result<PathBuf> {
        let file_path = self.work_dir.path().join(rel_path);
        
        // Ensure the file is within sandbox
        if !file_path.starts_with(self.work_dir.path()) {
            return Err(anyhow!("File path outside sandbox"));
        }
        
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&file_path, content)?;
        
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&file_path)?.permissions();
            perms.set_mode(0o600); // rw for owner only
            std::fs::set_permissions(&file_path, perms)?;
        }
        
        info!("Created file in sandbox: {:?}", file_path);
        Ok(file_path)
    }
    
    /// Copy a file into the sandbox
    pub fn copy_file_in<P: AsRef<Path>>(&self, src: P, dest_name: &str) -> Result<PathBuf> {
        let src_path = src.as_ref();
        let dest_path = self.work_dir.path().join(dest_name);
        
        // Validate source file
        if !src_path.exists() {
            return Err(anyhow!("Source file does not exist: {:?}", src_path));
        }
        
        // Check file size limit (100MB)
        let metadata = std::fs::metadata(src_path)?;
        if metadata.len() > 100 * 1024 * 1024 {
            return Err(anyhow!("File too large for sandbox (max 100MB)"));
        }
        
        std::fs::copy(src_path, &dest_path)?;
        
        info!("Copied file into sandbox: {:?} -> {:?}", src_path, dest_path);
        Ok(dest_path)
    }
    
    /// Extract a file from the sandbox
    pub fn extract_file<P: AsRef<Path>>(&self, src_name: &str, dest: P) -> Result<()> {
        let src_path = self.work_dir.path().join(src_name);
        let dest_path = dest.as_ref();
        
        // Ensure source is within sandbox
        if !src_path.starts_with(self.work_dir.path()) {
            return Err(anyhow!("Source path outside sandbox"));
        }
        
        if !src_path.exists() {
            return Err(anyhow!("Source file does not exist in sandbox"));
        }
        
        std::fs::copy(&src_path, dest_path)?;
        
        info!("Extracted file from sandbox: {:?} -> {:?}", src_path, dest_path);
        Ok(())
    }
    
    fn validate_path(&self, path: &Path) -> Result<()> {
        // Prevent access to sensitive system directories
        let forbidden_prefixes = [
            "/etc/",
            "/boot/",
            "/sys/",
            "/proc/",
            "/dev/",
            "/root/",
        ];
        
        let path_str = path.to_string_lossy();
        for prefix in &forbidden_prefixes {
            if path_str.starts_with(prefix) {
                return Err(anyhow!("Access to {} is forbidden", prefix));
            }
        }
        
        Ok(())
    }
    
    fn validate_command(&self, command: &str) -> Result<()> {
        // Only allow specific safe commands
        let allowed_commands = [
            "apt", "apt-get", "dpkg",
            "dnf", "yum", "rpm",
            "pacman",
            "snap",
            "flatpak",
            "wget", "curl",
            "tar", "gzip", "unzip",
            "sha256sum", "sha512sum",
            "gpg",
            "chmod", "chown",
            "mkdir", "cp", "mv", "rm",
        ];
        
        let cmd_name = std::path::Path::new(command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command);
        
        if !allowed_commands.contains(&cmd_name) {
            return Err(anyhow!("Command not allowed in sandbox: {}", cmd_name));
        }
        
        Ok(())
    }
    
    fn create_sandboxed_command(&self, command: &str, args: &[&str]) -> Result<Command> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.current_dir(self.work_dir.path());
        
        // Set restrictive environment
        cmd.env_clear();
        cmd.envs([
            ("PATH", "/usr/local/bin:/usr/bin:/bin:/sbin:/usr/sbin"),
            ("HOME", self.work_dir.path().to_str().unwrap()),
            ("TMPDIR", self.work_dir.path().to_str().unwrap()),
            ("USER", "sandbox"),
            ("SHELL", "/bin/sh"),
        ]);
        
        // Disable network if not allowed
        if !self.network_allowed {
            cmd.env("http_proxy", "127.0.0.1:1");
            cmd.env("https_proxy", "127.0.0.1:1");
            cmd.env("HTTP_PROXY", "127.0.0.1:1");
            cmd.env("HTTPS_PROXY", "127.0.0.1:1");
        }
        
        // Set up stdio
        cmd.stdin(Stdio::null());
        
        // Apply resource limits using unshare if available
        #[cfg(target_os = "linux")]
        {
            if Command::new("unshare").arg("--help").output().is_ok() {
                let mut unshare_cmd = Command::new("unshare");
                unshare_cmd.args([
                    "--pid", "--fork",
                    "--mount-proc",
                    "--user", "--map-root-user",
                ]);
                
                if !self.network_allowed {
                    unshare_cmd.arg("--net");
                }
                
                unshare_cmd.arg(command);
                unshare_cmd.args(args);
                return Ok(unshare_cmd);
            }
        }
        
        Ok(cmd)
    }
    
    /// Check if the sandbox is within resource limits
    pub fn check_resources(&self) -> Result<ResourceUsage> {
        let mut usage = ResourceUsage::default();
        
        // Calculate disk usage
        usage.disk_usage = self.calculate_disk_usage()?;
        
        // Memory usage would need to be tracked per process
        usage.memory_usage = 0; // Placeholder
        
        // Check limits
        if usage.disk_usage > 1024 * 1024 * 1024 { // 1GB limit
            return Err(anyhow!("Sandbox disk usage exceeded limit"));
        }
        
        Ok(usage)
    }
    
    fn calculate_disk_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        fn visit_dir(dir: &Path, total: &mut u64) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    visit_dir(&path, total)?;
                } else {
                    let metadata = entry.metadata()?;
                    *total += metadata.len();
                }
            }
            Ok(())
        }
        
        visit_dir(self.work_dir.path(), &mut total_size)?;
        Ok(total_size)
    }
}

#[derive(Debug, Default)]
pub struct ResourceUsage {
    pub disk_usage: u64,
    pub memory_usage: u64,
}

/// Network sandbox for isolating network operations
pub struct NetworkSandbox {
    allowed_hosts: Vec<String>,
    blocked_ports: Vec<u16>,
}

impl NetworkSandbox {
    pub fn new() -> Self {
        Self {
            allowed_hosts: vec![
                "packages.ubuntu.com".to_string(),
                "archive.ubuntu.com".to_string(),
                "download.fedoraproject.org".to_string(),
                "archlinux.org".to_string(),
                "snapcraft.io".to_string(),
                "flathub.org".to_string(),
            ],
            blocked_ports: vec![22, 23, 25, 53, 110, 143, 993, 995],
        }
    }
    
    pub fn allow_host(&mut self, host: &str) {
        self.allowed_hosts.push(host.to_string());
    }
    
    pub fn is_host_allowed(&self, host: &str) -> bool {
        self.allowed_hosts.iter().any(|allowed| {
            host == allowed || host.ends_with(&format!(".{}", allowed))
        })
    }
    
    pub fn is_port_blocked(&self, port: u16) -> bool {
        self.blocked_ports.contains(&port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new().unwrap();
        assert!(sandbox.work_dir().exists());
    }

    #[test]
    fn test_sandbox_file_operations() {
        let sandbox = Sandbox::new().unwrap();
        
        // Create a file
        let content = b"test content";
        let file_path = sandbox.create_file("test.txt", content).unwrap();
        assert!(file_path.exists());
        
        // Read it back
        let read_content = std::fs::read(&file_path).unwrap();
        assert_eq!(content, read_content.as_slice());
    }

    #[test]
    fn test_network_sandbox() {
        let mut net_sandbox = NetworkSandbox::new();
        
        // Test default allowed hosts
        assert!(net_sandbox.is_host_allowed("packages.ubuntu.com"));
        assert!(net_sandbox.is_host_allowed("security.ubuntu.com"));
        assert!(!net_sandbox.is_host_allowed("malicious.com"));
        
        // Test adding hosts
        net_sandbox.allow_host("example.com");
        assert!(net_sandbox.is_host_allowed("example.com"));
        assert!(net_sandbox.is_host_allowed("sub.example.com"));
        
        // Test blocked ports
        assert!(net_sandbox.is_port_blocked(22)); // SSH
        assert!(net_sandbox.is_port_blocked(25)); // SMTP
        assert!(!net_sandbox.is_port_blocked(80)); // HTTP
        assert!(!net_sandbox.is_port_blocked(443)); // HTTPS
    }

    #[test]
    fn test_command_validation() {
        let sandbox = Sandbox::new().unwrap();
        
        // Test allowed commands
        assert!(sandbox.validate_command("apt").is_ok());
        assert!(sandbox.validate_command("wget").is_ok());
        assert!(sandbox.validate_command("/usr/bin/sha256sum").is_ok());
        
        // Test forbidden commands
        assert!(sandbox.validate_command("nc").is_err());
        assert!(sandbox.validate_command("python").is_err());
        assert!(sandbox.validate_command("bash").is_err());
    }

    #[test]
    fn test_path_validation() {
        let sandbox = Sandbox::new().unwrap();
        
        // Test forbidden paths
        assert!(sandbox.validate_path(Path::new("/etc/passwd")).is_err());
        assert!(sandbox.validate_path(Path::new("/boot/vmlinuz")).is_err());
        assert!(sandbox.validate_path(Path::new("/root/.ssh")).is_err());
    }

    #[test]
    fn test_resource_checking() {
        let sandbox = Sandbox::new().unwrap();
        
        // Should succeed for empty sandbox
        let usage = sandbox.check_resources().unwrap();
        assert!(usage.disk_usage < 1024); // Should be very small
    }
}