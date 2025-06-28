use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use tracing::{info, warn, error};

/// Manages privilege escalation and dropping for secure operations
pub struct PrivilegeManager {
    original_uid: Option<u32>,
    original_gid: Option<u32>,
}

impl PrivilegeManager {
    pub fn new() -> Self {
        Self {
            original_uid: None,
            original_gid: None,
        }
    }
    
    /// Check if we're currently running as root
    pub fn is_root() -> bool {
        // SAFETY: getuid() is always safe to call and has no side effects
        // It only reads the current process's user ID from the kernel
        unsafe { libc::getuid() == 0 }
    }
    
    /// Check if we can escalate to root via sudo
    pub fn can_sudo() -> bool {
        Command::new("sudo")
            .arg("-n")
            .arg("true")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    
    /// Store current credentials before escalation
    pub fn store_credentials(&mut self) {
        // SAFETY: getuid() and getgid() are always safe to call
        // They only read process credentials from the kernel without side effects
        unsafe {
            self.original_uid = Some(libc::getuid());
            self.original_gid = Some(libc::getgid());
        }
        info!("Stored original credentials: uid={:?}, gid={:?}", 
              self.original_uid, self.original_gid);
    }
    
    /// Execute a command with elevated privileges
    pub fn execute_with_sudo(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        info!("Executing with sudo: {} {:?}", command, args);
        
        if !Self::can_sudo() {
            return Err(anyhow!("sudo access required but not available"));
        }
        
        let mut sudo_args = vec!["sudo", "-n", command];
        sudo_args.extend(args);
        
        let output = Command::new("sudo")
            .args(&sudo_args[1..]) // Skip the first "sudo"
            .output()
            .map_err(|e| anyhow!("Failed to execute sudo command: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Sudo command failed: {}", stderr);
            return Err(anyhow!("Command failed with status: {}", output.status));
        }
        
        Ok(output)
    }
    
    /// Drop privileges back to original user
    pub fn drop_privileges(&self) -> Result<()> {
        if !Self::is_root() {
            return Ok(()); // Already not root
        }
        
        let target_uid = self.original_uid.ok_or_else(|| {
            anyhow!("Cannot drop privileges: original UID not stored")
        })?;
        
        let target_gid = self.original_gid.ok_or_else(|| {
            anyhow!("Cannot drop privileges: original GID not stored")
        })?;
        
        info!("Dropping privileges to uid={}, gid={}", target_uid, target_gid);
        
        // Drop group privileges first
        // SAFETY: These system calls are the standard POSIX way to drop privileges
        // - setgid/setuid are designed for privilege management and are safe when used correctly
        // - setgroups is safe with a null pointer and count of 0 (clears supplementary groups)
        // - Error handling ensures we fail safely if privilege dropping fails
        unsafe {
            if libc::setgid(target_gid) != 0 {
                return Err(anyhow!("Failed to drop group privileges"));
            }
            
            // Clear supplementary groups
            if libc::setgroups(0, std::ptr::null()) != 0 {
                warn!("Failed to clear supplementary groups");
            }
            
            // Drop user privileges
            if libc::setuid(target_uid) != 0 {
                return Err(anyhow!("Failed to drop user privileges"));
            }
        }
        
        // Verify privileges were dropped
        // SAFETY: These are read-only system calls to verify privilege state
        // - getuid/geteuid/getgid/getegid only read current process credentials
        // - No side effects, always safe to call
        unsafe {
            if libc::getuid() != target_uid || libc::geteuid() != target_uid {
                return Err(anyhow!("Failed to verify user privilege drop"));
            }
            
            if libc::getgid() != target_gid || libc::getegid() != target_gid {
                return Err(anyhow!("Failed to verify group privilege drop"));
            }
        }
        
        info!("Successfully dropped privileges");
        Ok(())
    }
    
    /// Create a sandbox for package operations
    pub fn create_sandbox(&self, work_dir: &std::path::Path) -> Result<SandboxGuard> {
        info!("Creating sandbox in: {:?}", work_dir);
        
        // Create work directory if it doesn't exist
        std::fs::create_dir_all(work_dir)?;
        
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(work_dir)?.permissions();
            perms.set_mode(0o700); // rwx for owner only
            std::fs::set_permissions(work_dir, perms)?;
        }
        
        Ok(SandboxGuard {
            work_dir: work_dir.to_path_buf(),
        })
    }
    
    /// Execute a command in a restricted environment
    pub fn execute_sandboxed(&self, 
                            command: &str, 
                            args: &[&str], 
                            work_dir: &std::path::Path) -> Result<std::process::Output> {
        info!("Executing sandboxed command: {} {:?} in {:?}", command, args, work_dir);
        
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.current_dir(work_dir);
        
        // Set restrictive environment
        cmd.env_clear();
        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
        cmd.env("HOME", work_dir);
        cmd.env("TMPDIR", work_dir);
        
        // Limit resources if possible
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                // Set process limits
                let limit = libc::rlimit {
                    rlim_cur: 64 * 1024 * 1024, // 64MB memory limit
                    rlim_max: 64 * 1024 * 1024,
                };
                libc::setrlimit(libc::RLIMIT_AS, &limit);
                
                // Limit CPU time (30 seconds)
                let time_limit = libc::rlimit {
                    rlim_cur: 30,
                    rlim_max: 30,
                };
                libc::setrlimit(libc::RLIMIT_CPU, &time_limit);
                
                Ok(())
            });
        }
        
        let output = cmd.output()
            .map_err(|e| anyhow!("Failed to execute sandboxed command: {}", e))?;
        
        Ok(output)
    }
    
    /// Validate that current process has minimal required privileges
    pub fn validate_minimal_privileges() -> Result<()> {
        // Check that we're not running as root unless necessary
        if Self::is_root() {
            warn!("Running as root - consider using privilege dropping");
        }
        
        // Verify umask is restrictive
        #[cfg(unix)]
        // SAFETY: umask() is safe to call - it only modifies file creation permissions
        // We temporarily set a restrictive umask to read the current value, then restore it
        // This is a standard pattern for checking umask without permanently changing it
        unsafe {
            let umask = libc::umask(0o077); // Set restrictive umask
            libc::umask(umask); // Restore original
            
            if umask > 0o077 {
                warn!("umask is not restrictive enough: {:o}", umask);
            }
        }
        
        Ok(())
    }
}

/// RAII guard for sandbox cleanup
pub struct SandboxGuard {
    work_dir: std::path::PathBuf,
}

impl Drop for SandboxGuard {
    fn drop(&mut self) {
        info!("Cleaning up sandbox: {:?}", self.work_dir);
        
        if let Err(e) = std::fs::remove_dir_all(&self.work_dir) {
            error!("Failed to cleanup sandbox: {}", e);
        }
    }
}

impl SandboxGuard {
    pub fn work_dir(&self) -> &std::path::Path {
        &self.work_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_privilege_manager_creation() {
        let pm = PrivilegeManager::new();
        assert!(pm.original_uid.is_none());
        assert!(pm.original_gid.is_none());
    }

    #[test]
    fn test_store_credentials() {
        let mut pm = PrivilegeManager::new();
        pm.store_credentials();
        assert!(pm.original_uid.is_some());
        assert!(pm.original_gid.is_some());
    }

    #[test]
    fn test_can_sudo() {
        // This test depends on the environment, so we just check it doesn't panic
        let _can_sudo = PrivilegeManager::can_sudo();
    }

    #[test]
    fn test_sandbox_creation() {
        let pm = PrivilegeManager::new();
        let temp_dir = TempDir::new().unwrap();
        let sandbox_dir = temp_dir.path().join("sandbox");
        
        let _guard = pm.create_sandbox(&sandbox_dir).unwrap();
        assert!(sandbox_dir.exists());
    }

    #[test]
    fn test_minimal_privileges_validation() {
        // Should not panic
        let _ = PrivilegeManager::validate_minimal_privileges();
    }
}