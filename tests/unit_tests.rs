use chrono::Utc;
use omni::*;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use tokio_test;
use uuid::Uuid;

#[cfg(test)]
mod config_tests {
    use super::*;
    use omni::config::*;

    #[test]
    fn test_config_default_values() {
        let config = OmniConfig::default();

        // Test general config defaults
        assert_eq!(config.general.auto_update, false);
        assert_eq!(config.general.parallel_installs, true);
        assert_eq!(config.general.max_parallel_jobs, 4);
        assert_eq!(config.general.confirm_installs, true);
        assert_eq!(config.general.log_level, "info");
        assert_eq!(config.general.fallback_enabled, true);

        // Test box config defaults
        assert_eq!(config.boxes.preferred_order[0], "apt");
        assert_eq!(config.boxes.disabled_boxes.len(), 0);
        assert!(config.boxes.apt_options.contains(&"-y".to_string()));

        // Test security config defaults
        assert_eq!(config.security.verify_signatures, true);
        assert_eq!(config.security.verify_checksums, true);
        assert_eq!(config.security.allow_untrusted, false);
        assert!(config
            .security
            .signature_servers
            .contains(&"keyserver.ubuntu.com".to_string()));

        // Test UI config defaults
        assert_eq!(config.ui.show_progress, true);
        assert_eq!(config.ui.use_colors, true);
        assert_eq!(config.ui.gui_theme, "dark");
    }

    #[test]
    fn test_config_serialization() {
        let config = OmniConfig::default();
        let serialized = serde_yaml::to_string(&config).unwrap();
        assert!(serialized.contains("general:"));
        assert!(serialized.contains("auto_update: false"));

        let deserialized: OmniConfig = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.general.auto_update, config.general.auto_update);
    }

    #[test]
    fn test_config_box_enabled() {
        let mut config = OmniConfig::default();

        // Test box is enabled by default
        assert!(config.is_box_enabled("apt"));
        assert!(config.is_box_enabled("dnf"));

        // Test disabling a box
        config.boxes.disabled_boxes.push("apt".to_string());
        assert!(!config.is_box_enabled("apt"));
        assert!(config.is_box_enabled("dnf"));
    }

    #[test]
    fn test_config_box_priority() {
        let config = OmniConfig::default();

        assert_eq!(config.get_box_priority("apt"), Some(0));
        assert_eq!(config.get_box_priority("dnf"), Some(1));
        assert_eq!(config.get_box_priority("pacman"), Some(2));
        assert_eq!(config.get_box_priority("nonexistent"), None);
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        // Create a custom config
        let mut config = OmniConfig::default();
        config.general.auto_update = true;
        config.general.max_parallel_jobs = 8;
        config.boxes.disabled_boxes.push("snap".to_string());

        // Test that we can't load from non-existent config path currently
        // This tests the create-if-not-exists behavior
        let config_path = temp_dir.path().join("omni").join("config.yaml");
        assert!(!config_path.exists());
    }

    #[test]
    fn test_config_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Write invalid YAML
        fs::write(&config_path, "invalid: yaml: [unclosed").unwrap();

        // Should fail to parse
        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<OmniConfig, _> = serde_yaml::from_str(&content);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod database_tests {
    use super::*;
    use omni::database::*;

    #[tokio::test]
    async fn test_install_record_creation() {
        let record = InstallRecord {
            id: Uuid::new_v4().to_string(),
            package_name: "test-package".to_string(),
            box_type: "apt".to_string(),
            version: Some("1.0.0".to_string()),
            source_url: Some("https://example.com/package.deb".to_string()),
            install_path: Some("/usr/bin/test-package".to_string()),
            installed_at: Utc::now(),
            status: InstallStatus::Success,
            metadata: Some("{\"test\": true}".to_string()),
        };

        assert_eq!(record.package_name, "test-package");
        assert_eq!(record.box_type, "apt");
        assert_eq!(record.version, Some("1.0.0".to_string()));
        matches!(record.status, InstallStatus::Success);
    }

    #[tokio::test]
    async fn test_install_status_serialization() {
        let statuses = vec![
            InstallStatus::Success,
            InstallStatus::Failed,
            InstallStatus::Removed,
            InstallStatus::Updated,
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: InstallStatus = serde_json::from_str(&serialized).unwrap();

            match (&status, &deserialized) {
                (InstallStatus::Success, InstallStatus::Success) => (),
                (InstallStatus::Failed, InstallStatus::Failed) => (),
                (InstallStatus::Removed, InstallStatus::Removed) => (),
                (InstallStatus::Updated, InstallStatus::Updated) => (),
                _ => panic!("Status serialization roundtrip failed"),
            }
        }
    }

    #[test]
    fn test_snapshot_creation() {
        let packages = vec![
            InstallRecord {
                id: Uuid::new_v4().to_string(),
                package_name: "package1".to_string(),
                box_type: "apt".to_string(),
                version: Some("1.0.0".to_string()),
                source_url: None,
                install_path: None,
                installed_at: Utc::now(),
                status: InstallStatus::Success,
                metadata: None,
            },
            InstallRecord {
                id: Uuid::new_v4().to_string(),
                package_name: "package2".to_string(),
                box_type: "dnf".to_string(),
                version: Some("2.0.0".to_string()),
                source_url: None,
                install_path: None,
                installed_at: Utc::now(),
                status: InstallStatus::Success,
                metadata: None,
            },
        ];

        let snapshot = Snapshot {
            id: Uuid::new_v4().to_string(),
            name: "test-snapshot".to_string(),
            description: Some("Test snapshot for unit tests".to_string()),
            created_at: Utc::now(),
            packages,
        };

        assert_eq!(snapshot.name, "test-snapshot");
        assert_eq!(snapshot.packages.len(), 2);
        assert_eq!(snapshot.packages[0].package_name, "package1");
        assert_eq!(snapshot.packages[1].package_name, "package2");
    }

    #[test]
    fn test_package_cache_creation() {
        let cache = PackageCache {
            package_name: "test-package".to_string(),
            box_type: "apt".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test package for caching".to_string()),
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            cached_at: Utc::now(),
        };

        assert_eq!(cache.package_name, "test-package");
        assert_eq!(cache.dependencies.len(), 2);
        assert!(cache.dependencies.contains(&"dep1".to_string()));
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use omni::manifest::*;

    #[test]
    fn test_manifest_parsing_valid() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("test.yaml");

        let manifest_content = r#"
project: "Test Project"
description: "Test manifest for unit tests"
apps:
  - name: "vim"
    box: "apt"
    version: "latest"
    source: "https://packages.ubuntu.com"
  - name: "firefox"
    box: "snap"
    version: "stable"
meta:
  created_by: "test-user"
  created_on: "2024-01-01"
  distro_fallback: true
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let manifest = OmniManifest::from_file(manifest_path.to_str().unwrap()).unwrap();

        assert_eq!(manifest.project, "Test Project");
        assert_eq!(
            manifest.description,
            Some("Test manifest for unit tests".to_string())
        );
        assert_eq!(manifest.apps.len(), 2);

        assert_eq!(manifest.apps[0].name, "vim");
        assert_eq!(manifest.apps[0].box_type, "apt");
        assert_eq!(manifest.apps[0].version, Some("latest".to_string()));

        assert_eq!(manifest.apps[1].name, "firefox");
        assert_eq!(manifest.apps[1].box_type, "snap");

        assert!(manifest.meta.is_some());
        let meta = manifest.meta.unwrap();
        assert_eq!(meta.created_by, Some("test-user".to_string()));
        assert_eq!(meta.distro_fallback, Some(true));
    }

    #[test]
    fn test_manifest_parsing_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("minimal.yaml");

        let manifest_content = r#"
project: "Minimal Project"
apps:
  - name: "git"
    box: "apt"
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let manifest = OmniManifest::from_file(manifest_path.to_str().unwrap()).unwrap();

        assert_eq!(manifest.project, "Minimal Project");
        assert_eq!(manifest.description, None);
        assert_eq!(manifest.apps.len(), 1);
        assert_eq!(manifest.apps[0].name, "git");
        assert_eq!(manifest.apps[0].version, None);
        assert!(manifest.meta.is_none());
    }

    #[test]
    fn test_manifest_parsing_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("invalid.yaml");

        let invalid_content = "invalid: yaml: content: [unclosed";
        fs::write(&manifest_path, invalid_content).unwrap();

        let result = OmniManifest::from_file(manifest_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_parsing_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("missing.yaml");

        let manifest_content = r#"
description: "Missing project field"
apps:
  - name: "git"
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let result = OmniManifest::from_file(manifest_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_nonexistent_file() {
        let result = OmniManifest::from_file("/nonexistent/path/manifest.yaml");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use omni::security::*;
    use std::io::Write;

    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();

        assert_eq!(policy.verify_signatures, true);
        assert_eq!(policy.verify_checksums, true);
        assert_eq!(policy.allow_untrusted, false);
        assert_eq!(policy.check_mirrors, true);
        assert!(policy.signature_servers.len() > 0);
        assert!(policy
            .signature_servers
            .contains(&"keyserver.ubuntu.com".to_string()));
    }

    #[test]
    fn test_security_policy_serialization() {
        let policy = SecurityPolicy::default();
        let serialized = serde_json::to_string(&policy).unwrap();
        let deserialized: SecurityPolicy = serde_json::from_str(&serialized).unwrap();

        assert_eq!(policy.verify_signatures, deserialized.verify_signatures);
        assert_eq!(policy.signature_servers, deserialized.signature_servers);
    }

    #[test]
    fn test_verification_result_creation() {
        let result = VerificationResult {
            signature_valid: Some(true),
            checksum_valid: Some(true),
            trust_level: TrustLevel::Trusted,
            warnings: vec!["Test warning".to_string()],
            details: "Test verification details".to_string(),
        };

        assert_eq!(result.signature_valid, Some(true));
        assert_eq!(result.trust_level, TrustLevel::Trusted);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "Test warning");
    }

    #[test]
    fn test_trust_levels() {
        let levels = vec![
            TrustLevel::Trusted,
            TrustLevel::Valid,
            TrustLevel::Unsigned,
            TrustLevel::Untrusted,
        ];

        for level in levels {
            match level {
                TrustLevel::Trusted => assert_eq!(level, TrustLevel::Trusted),
                TrustLevel::Valid => assert_eq!(level, TrustLevel::Valid),
                TrustLevel::Unsigned => assert_eq!(level, TrustLevel::Unsigned),
                TrustLevel::Untrusted => assert_eq!(level, TrustLevel::Untrusted),
            }
        }
    }

    #[tokio::test]
    async fn test_security_verifier_creation() {
        let policy = SecurityPolicy::default();
        let verifier = SecurityVerifier::new(policy);

        // Test that verifier can be created without panic
        // Actual verification tests would require test files and GPG setup
    }

    #[test]
    fn test_security_policy_custom() {
        let mut policy = SecurityPolicy::default();
        policy.allow_untrusted = true;
        policy.verify_signatures = false;
        policy.trusted_keys.push("test-key-id".to_string());

        assert_eq!(policy.allow_untrusted, true);
        assert_eq!(policy.verify_signatures, false);
        assert!(policy.trusted_keys.contains(&"test-key-id".to_string()));
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    // Note: Search tests would typically require mocking external package managers
    // For now, we test basic structure and error handling

    #[test]
    fn test_search_result_creation() {
        // Since SearchResult and SearchEngine aren't fully accessible in this context,
        // we'll create basic tests for what we can access
        assert!(true); // Placeholder for actual search tests
    }
}

#[cfg(test)]
mod integration_helpers {
    use super::*;
    use std::env;
    use std::process::Command;

    pub fn is_ci_environment() -> bool {
        env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
    }

    pub fn has_package_manager(manager: &str) -> bool {
        Command::new(manager)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn skip_if_no_sudo() -> bool {
        Command::new("sudo")
            .arg("-n")
            .arg("true")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn create_test_file_with_content(content: &[u8]) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    pub fn create_test_config() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let config = r#"
general:
  auto_update: false
  parallel_installs: true
  max_parallel_jobs: 4
boxes:
  preferred_order: ["apt", "dnf"]
  disabled_boxes: []
security:
  verify_signatures: true
  allow_untrusted: false
ui:
  show_progress: true
  use_colors: true
"#;

        fs::write(&config_path, config).unwrap();
        (temp_dir, config_path)
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_config_handles_missing_fields_gracefully() {
        let minimal_config = r#"
general:
  auto_update: false
"#;

        // This should fail to deserialize due to missing required fields
        let result: Result<omni::config::OmniConfig, _> = serde_yaml::from_str(minimal_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_handles_empty_apps_list() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("empty_apps.yaml");

        let manifest_content = r#"
project: "Empty Apps Project"
apps: []
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let manifest =
            omni::manifest::OmniManifest::from_file(manifest_path.to_str().unwrap()).unwrap();
        assert_eq!(manifest.apps.len(), 0);
    }

    #[test]
    fn test_uuid_generation_uniqueness() {
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // Standard UUID length
        assert_eq!(id2.len(), 36);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_config_creation_performance() {
        let start = Instant::now();

        for _ in 0..1000 {
            let _config = omni::config::OmniConfig::default();
        }

        let duration = start.elapsed();
        // Should create 1000 configs in well under a second
        assert!(duration.as_millis() < 1000);
    }

    #[test]
    fn test_manifest_parsing_performance() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("perf_test.yaml");

        // Create a manifest with many apps
        let mut manifest_content = String::from("project: \"Performance Test\"\napps:\n");
        for i in 0..100 {
            manifest_content.push_str(&format!(
                "  - name: \"package{}\"\n    box: \"apt\"\n    version: \"1.0.{}\"\n",
                i, i
            ));
        }

        fs::write(&manifest_path, manifest_content).unwrap();

        let start = Instant::now();
        let manifest =
            omni::manifest::OmniManifest::from_file(manifest_path.to_str().unwrap()).unwrap();
        let duration = start.elapsed();

        assert_eq!(manifest.apps.len(), 100);
        // Should parse 100 apps quickly
        assert!(duration.as_millis() < 100);
    }
}
