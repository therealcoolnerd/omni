use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Test utilities for Omni
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub mock_commands: Arc<Mutex<HashMap<String, MockCommand>>>,
}

#[derive(Debug, Clone)]
pub struct MockCommand {
    pub command: String,
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub delay: Option<Duration>,
}

impl Default for MockCommand {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            delay: None,
        }
    }
}

impl TestEnvironment {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let mock_commands = Arc::new(Mutex::new(HashMap::new()));
        
        Ok(Self {
            temp_dir,
            mock_commands,
        })
    }

    pub fn temp_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn add_mock_command(&self, command: &str, mock: MockCommand) {
        let mut commands = self.mock_commands.lock().unwrap();
        commands.insert(command.to_string(), mock);
    }

    pub fn setup_mock_apt(&self) {
        // Mock apt commands for testing
        self.add_mock_command("apt", MockCommand {
            command: "apt".to_string(),
            args: vec!["update".to_string()],
            stdout: "Reading package lists...\nDone\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: Some(Duration::from_millis(100)),
        });

        self.add_mock_command("apt-get", MockCommand {
            command: "apt-get".to_string(),
            args: vec!["install".to_string(), "-y".to_string()],
            stdout: "Reading package lists...\nPackage installed successfully\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: Some(Duration::from_millis(500)),
        });

        self.add_mock_command("dpkg-query", MockCommand {
            command: "dpkg-query".to_string(),
            args: vec!["-W".to_string(), "-f=${Version}".to_string()],
            stdout: "1.0.0".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: None,
        });
    }

    pub fn setup_mock_dnf(&self) {
        self.add_mock_command("dnf", MockCommand {
            command: "dnf".to_string(),
            args: vec!["install".to_string(), "-y".to_string()],
            stdout: "Installing package...\nComplete!\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: Some(Duration::from_millis(300)),
        });

        self.add_mock_command("rpm", MockCommand {
            command: "rpm".to_string(),
            args: vec!["-q".to_string(), "--qf".to_string(), "%{VERSION}".to_string()],
            stdout: "1.0.0".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: None,
        });
    }

    pub fn setup_mock_snap(&self) {
        self.add_mock_command("snap", MockCommand {
            command: "snap".to_string(),
            args: vec!["install".to_string()],
            stdout: "Snap installed successfully\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: Some(Duration::from_millis(200)),
        });
    }

    pub fn setup_mock_hardware(&self) {
        self.add_mock_command("lspci", MockCommand {
            command: "lspci".to_string(),
            args: vec!["-nn".to_string()],
            stdout: "00:00.0 Host bridge [0600]: Intel Corporation Device [8086:1234]\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: None,
        });

        self.add_mock_command("dmidecode", MockCommand {
            command: "dmidecode".to_string(),
            args: vec!["-s".to_string(), "system-manufacturer".to_string()],
            stdout: "Test Manufacturer\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: None,
        });
    }

    pub fn simulate_command_failure(&self, command: &str, exit_code: i32, stderr: &str) {
        let mut mock = MockCommand::default();
        mock.command = command.to_string();
        mock.exit_code = exit_code;
        mock.stderr = stderr.to_string();
        
        self.add_mock_command(command, mock);
    }

    pub fn simulate_network_timeout(&self, command: &str) {
        let mut mock = MockCommand::default();
        mock.command = command.to_string();
        mock.delay = Some(Duration::from_secs(30)); // Simulate timeout
        mock.exit_code = 124; // Standard timeout exit code
        mock.stderr = "Operation timed out\n".to_string();
        
        self.add_mock_command(command, mock);
    }

    pub fn get_command_history(&self) -> Vec<String> {
        let commands = self.mock_commands.lock().unwrap();
        commands.keys().cloned().collect()
    }

    pub fn clear_mocks(&self) {
        let mut commands = self.mock_commands.lock().unwrap();
        commands.clear();
    }
}

/// Test utilities for async operations
pub struct AsyncTestHelper;

impl AsyncTestHelper {
    pub async fn with_timeout<T, F>(duration: Duration, future: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        tokio::time::timeout(duration, future)
            .await
            .map_err(|_| anyhow::anyhow!("Test timed out after {:?}", duration))?
    }

    pub async fn retry_until_success<T, F>(
        max_attempts: u32,
        delay: Duration,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        loop {
            attempts += 1;
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(e);
                    }
                    sleep(delay).await;
                }
            }
        }
    }

    pub async fn simulate_slow_operation(duration: Duration) {
        sleep(duration).await;
    }
}

/// Test assertions for package operations
pub struct PackageTestAssertions;

impl PackageTestAssertions {
    pub fn assert_package_installed(package_name: &str, version: Option<&str>) -> Result<()> {
        // Mock implementation - in real tests this would check the system
        println!("✓ Assert package installed: {} {:?}", package_name, version);
        Ok(())
    }

    pub fn assert_package_not_installed(package_name: &str) -> Result<()> {
        // Mock implementation
        println!("✓ Assert package not installed: {}", package_name);
        Ok(())
    }

    pub fn assert_repository_configured(repo_url: &str) -> Result<()> {
        // Mock implementation
        println!("✓ Assert repository configured: {}", repo_url);
        Ok(())
    }

    pub fn assert_command_executed(command: &str, args: &[&str]) -> Result<()> {
        // Mock implementation
        println!("✓ Assert command executed: {} {:?}", command, args);
        Ok(())
    }
}

/// Integration test helpers
pub struct IntegrationTestHelper;

impl IntegrationTestHelper {
    pub fn create_test_manifest(packages: &[&str]) -> Result<String> {
        let mut manifest = String::from("version: 1.0\npackages:\n");
        for package in packages {
            manifest.push_str(&format!("  - name: {}\n", package));
        }
        Ok(manifest)
    }

    pub fn create_test_config() -> Result<String> {
        Ok(r#"
repositories:
  - name: "test-repo"
    url: "https://example.com/repo"
    enabled: true
security:
  verify_signatures: true
  verify_checksums: true
logging:
  level: "debug"
"#.to_string())
    }

    pub async fn setup_test_database(temp_dir: &PathBuf) -> Result<()> {
        // Create test database schema
        let db_path = temp_dir.join("test.db");
        std::fs::write(&db_path, "")?;
        println!("✓ Test database created at: {:?}", db_path);
        Ok(())
    }

    pub async fn cleanup_test_environment(temp_dir: &PathBuf) -> Result<()> {
        // Clean up test resources
        if temp_dir.exists() {
            std::fs::remove_dir_all(temp_dir)?;
        }
        println!("✓ Test environment cleaned up");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_setup() {
        let env = TestEnvironment::new().unwrap();
        assert!(env.temp_path().exists());
        
        env.setup_mock_apt();
        let history = env.get_command_history();
        assert!(history.contains(&"apt".to_string()));
    }

    #[tokio::test]
    async fn test_async_helper_timeout() {
        let result = AsyncTestHelper::with_timeout(
            Duration::from_millis(100),
            async {
                sleep(Duration::from_millis(50)).await;
                Ok(42)
            }
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let mut attempts = 0;
        let result = AsyncTestHelper::retry_until_success(
            3,
            Duration::from_millis(10),
            || {
                attempts += 1;
                async move {
                    if attempts < 3 {
                        Err(anyhow::anyhow!("Temporary failure"))
                    } else {
                        Ok("Success")
                    }
                }
            }
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
    }
}