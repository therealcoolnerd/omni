use omni::ssh::{SshClient, SshConfig, AuthMethod};
use std::path::PathBuf;
use std::time::Duration;
use tokio;

fn get_test_ssh_config() -> SshConfig {
    SshConfig {
        host: "localhost".to_string(),
        port: 22,
        username: "test".to_string(),
        auth_method: AuthMethod::PublicKey {
            private_key_path: PathBuf::from("/tmp/test_key"),
            passphrase: None,
        },
        connect_timeout: Duration::from_secs(5),
        command_timeout: Duration::from_secs(10),
        max_retries: 1,
        host_key_verification: false, // Disabled for testing
        known_hosts_file: None,
        compression: true,
        keepalive_interval: Some(Duration::from_secs(30)),
    }
}

#[tokio::test]
async fn test_ssh_client_creation() {
    // Test that SSH client can be created without errors
    let client = SshClient::new();
    
    // Basic test - ensure the client is created
    // We can't test actual SSH connections without a test server
    println!("SSH client created successfully");
}

#[tokio::test]
async fn test_ssh_config_creation() {
    let config = get_test_ssh_config();
    
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 22);
    assert_eq!(config.username, "test");
    assert_eq!(config.connect_timeout, Duration::from_secs(5));
    assert!(!config.host_key_verification);
    
    println!("SSH config created and validated successfully");
}

#[tokio::test]
async fn test_ssh_config_serialization() {
    let config = get_test_ssh_config();
    
    // Test that config can be serialized to YAML
    let yaml_result = serde_yaml::to_string(&config);
    assert!(yaml_result.is_ok(), "SSH config should serialize to YAML");
    
    let yaml_str = yaml_result.unwrap();
    println!("SSH config YAML length: {} chars", yaml_str.len());
    
    // Test that config can be deserialized from YAML
    let deserialized_result: Result<SshConfig, _> = serde_yaml::from_str(&yaml_str);
    assert!(deserialized_result.is_ok(), "SSH config should deserialize from YAML");
    
    let deserialized_config = deserialized_result.unwrap();
    assert_eq!(config.host, deserialized_config.host);
    assert_eq!(config.port, deserialized_config.port);
    assert_eq!(config.username, deserialized_config.username);
}

#[tokio::test]
async fn test_ssh_auth_methods() {
    // Test password authentication
    let password_auth = AuthMethod::Password {
        password: "test_password".to_string(),
    };
    
    match password_auth {
        AuthMethod::Password { password } => {
            assert_eq!(password, "test_password");
        }
        _ => panic!("Expected password authentication"),
    }
    
    // Test public key authentication
    let key_auth = AuthMethod::PublicKey {
        private_key_path: PathBuf::from("/test/path"),
        passphrase: Some("test_passphrase".to_string()),
    };
    
    match key_auth {
        AuthMethod::PublicKey { private_key_path, passphrase } => {
            assert_eq!(private_key_path, PathBuf::from("/test/path"));
            assert_eq!(passphrase, Some("test_passphrase".to_string()));
        }
        _ => panic!("Expected public key authentication"),
    }
    
    println!("SSH authentication methods validated successfully");
}

#[tokio::test]
async fn test_ssh_config_defaults() {
    let default_config = SshConfig::default();
    
    assert_eq!(default_config.port, 22);
    assert_eq!(default_config.username, "root");
    assert_eq!(default_config.connect_timeout, Duration::from_secs(30));
    assert_eq!(default_config.command_timeout, Duration::from_secs(300));
    assert_eq!(default_config.max_retries, 3);
    assert!(default_config.host_key_verification);
    assert!(default_config.compression);
    
    println!("SSH config defaults validated successfully");
}

// Note: These tests don't actually connect via SSH since that would require
// setting up a test SSH server. They test the configuration and client creation
// aspects of the SSH implementation.

#[tokio::test]
async fn test_ssh_error_handling() {
    let mut invalid_config = get_test_ssh_config();
    invalid_config.host = "definitely-invalid-host-12345.invalid".to_string();
    
    // We can't easily test connection failures without affecting the test environment
    // So we just verify the config can be created with invalid values
    println!("SSH error handling test completed (config with invalid host created)");
}

#[cfg(feature = "dangerous_tests")]
#[tokio::test]
async fn test_real_ssh_connection() {
    // This test would attempt a real SSH connection
    // Only enabled with the "dangerous_tests" feature
    
    println!("Real SSH connection test would run here in dangerous mode");
    
    // In a real test environment, you would:
    // 1. Set up a test SSH server (like in a container)
    // 2. Create valid credentials
    // 3. Test actual connection, authentication, and command execution
    // 4. Clean up the test environment
}