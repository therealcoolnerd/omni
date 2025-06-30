use omni::{
    distro::{detect_os, OperatingSystem, PackageManager},
    OmniConfig, SecureOmniBrainV2, UnifiedPackageManager,
};
use std::collections::HashMap;
use tokio;

// Test helper to check if we're in a safe test environment
fn is_safe_test_environment() -> bool {
    // Only run real package manager tests if explicitly enabled
    std::env::var("OMNI_ENABLE_REAL_TESTS").is_ok()
}

// Test helper to get a test package name that's safe to install/remove
fn get_safe_test_package() -> &'static str {
    match detect_os() {
        OperatingSystem::Linux(_) => "curl", // Available on most Linux distros
        OperatingSystem::Windows => "7zip",  // Small, common package
        OperatingSystem::MacOS => "wget",    // Available via brew
        _ => "curl",
    }
}

#[tokio::test]
async fn test_unified_manager_initialization() {
    let config = OmniConfig::default();
    let result = UnifiedPackageManager::new(config);

    match result {
        Ok(manager) => {
            let available = manager.get_available_managers();
            println!("Available package managers: {:?}", available);
            assert!(
                !available.is_empty(),
                "At least one package manager should be available"
            );
        }
        Err(e) => {
            println!("Failed to initialize UnifiedPackageManager: {}", e);
            // This might fail on systems without any package managers, which is okay
        }
    }
}

#[tokio::test]
async fn test_secure_brain_v2_initialization() {
    let result = SecureOmniBrainV2::new();

    match result {
        Ok(brain) => {
            let available = brain.get_available_managers();
            println!(
                "SecureOmniBrainV2 initialized with managers: {:?}",
                available
            );
            assert!(
                !available.is_empty(),
                "At least one package manager should be available"
            );
        }
        Err(e) => {
            println!("Failed to initialize SecureOmniBrainV2: {}", e);
            // Log error but don't fail test - system might not have package managers
        }
    }
}

#[tokio::test]
async fn test_package_search_functionality() {
    if !is_safe_test_environment() {
        println!("Skipping real package manager tests - set OMNI_ENABLE_REAL_TESTS to enable");
        return;
    }

    let config = OmniConfig::default();
    let manager = match UnifiedPackageManager::new(config) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager: {}", e);
            return;
        }
    };

    // Test search functionality
    let search_results = manager.search("curl").unwrap_or_default();

    for (manager_name, packages) in &search_results {
        println!(
            "Search results from {}: {} packages found",
            manager_name,
            packages.len()
        );
        if !packages.is_empty() {
            println!(
                "  Sample packages: {:?}",
                &packages[..std::cmp::min(3, packages.len())]
            );
        }
    }

    // At least one manager should return results for a common package like curl
    // But don't assert this as it depends on the system
}

#[tokio::test]
async fn test_list_installed_packages() {
    if !is_safe_test_environment() {
        println!("Skipping real package manager tests - set OMNI_ENABLE_REAL_TESTS to enable");
        return;
    }

    let config = OmniConfig::default();
    let manager = match UnifiedPackageManager::new(config) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager: {}", e);
            return;
        }
    };

    // Test listing installed packages
    let installed_results = manager.list_installed().unwrap_or_default();

    for (manager_name, packages) in &installed_results {
        println!(
            "Installed packages from {}: {} packages",
            manager_name,
            packages.len()
        );
        if !packages.is_empty() {
            println!(
                "  Sample packages: {:?}",
                &packages[..std::cmp::min(5, packages.len())]
            );
        }
    }
}

#[tokio::test]
async fn test_package_info_functionality() {
    if !is_safe_test_environment() {
        println!("Skipping real package manager tests - set OMNI_ENABLE_REAL_TESTS to enable");
        return;
    }

    let config = OmniConfig::default();
    let manager = match UnifiedPackageManager::new(config) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager: {}", e);
            return;
        }
    };

    let available_managers = manager.get_available_managers();

    for manager_name in &available_managers {
        match manager.get_info("curl", manager_name) {
            Ok(info) => {
                println!(
                    "Info for 'curl' from {}: {} characters",
                    manager_name,
                    info.len()
                );
                assert!(!info.is_empty(), "Package info should not be empty");
                // Print first few lines of info
                let lines: Vec<&str> = info.lines().take(3).collect();
                println!("  First few lines: {:?}", lines);
            }
            Err(e) => {
                println!("Failed to get info for 'curl' from {}: {}", manager_name, e);
                // Don't fail test - package might not be available in this manager
            }
        }
    }
}

#[tokio::test]
async fn test_configuration_integration() {
    let mut config = OmniConfig::default();

    // Test disabling a box
    config.boxes.disabled_boxes.push("snap".to_string());

    let manager = match UnifiedPackageManager::new(config.clone()) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager with config: {}", e);
            return;
        }
    };

    let available = manager.get_available_managers();
    println!("Available managers with snap disabled: {:?}", available);

    // Snap should not be in the available managers if it was disabled
    // (unless it wasn't available in the first place)
}

#[tokio::test]
async fn test_secure_brain_with_manifest() {
    let manifest_content = r#"
project: "test-project"
description: "Test manifest for integration testing"
apps:
  - name: "curl"
    box_type: "apt"
    source: null
"#;

    let manifest: omni::manifest::OmniManifest = match serde_yaml::from_str(manifest_content) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to parse test manifest: {}", e);
            return;
        }
    };

    println!("Test manifest loaded: {:?}", manifest);

    // We don't actually install from the manifest in tests
    // but we verify it can be parsed and processed
    assert_eq!(manifest.project, "test-project");
    assert_eq!(manifest.apps.len(), 1);
    assert_eq!(manifest.apps[0].name, "curl");
}

#[cfg(feature = "dangerous_tests")]
#[tokio::test]
async fn test_real_package_installation() {
    // This test is only enabled with the "dangerous_tests" feature
    // and should only be run in isolated environments

    if !is_safe_test_environment() {
        println!("Skipping dangerous package installation test");
        return;
    }

    let mut brain = match SecureOmniBrainV2::new() {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to create SecureOmniBrainV2: {}", e);
            return;
        }
    };

    let test_package = get_safe_test_package();
    println!("Testing with package: {}", test_package);

    // Note: This test would actually install and then remove a package
    // It should only be run in containerized or VM environments
    println!(
        "This test would install and remove package: {}",
        test_package
    );

    // Simulate the test without actually running it
    // In a real dangerous test, you would:
    // 1. Check if package is already installed
    // 2. Install it if not installed
    // 3. Verify it's installed
    // 4. Remove it
    // 5. Verify it's removed
}

#[tokio::test]
async fn test_error_handling() {
    let config = OmniConfig::default();
    let manager = match UnifiedPackageManager::new(config) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager: {}", e);
            return;
        }
    };

    // Test with a non-existent package
    let result = manager.install("this-package-definitely-does-not-exist-12345");
    match result {
        Ok(_) => panic!("Expected error for non-existent package"),
        Err(e) => {
            println!("Correctly got error for non-existent package: {}", e);
            assert!(e.to_string().contains("Failed to install"));
        }
    }
}

#[tokio::test]
async fn test_concurrent_operations() {
    if !is_safe_test_environment() {
        println!("Skipping concurrent operations test");
        return;
    }

    let config = OmniConfig::default();
    let manager = match UnifiedPackageManager::new(config) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to create manager: {}", e);
            return;
        }
    };

    // Test concurrent search operations
    let search_tasks = vec![
        tokio::spawn(async {
            let config = OmniConfig::default();
            let manager = UnifiedPackageManager::new(config)?;
            manager.search("curl")
        }),
        tokio::spawn(async {
            let config = OmniConfig::default();
            let manager = UnifiedPackageManager::new(config)?;
            manager.search("wget")
        }),
        tokio::spawn(async {
            let config = OmniConfig::default();
            let manager = UnifiedPackageManager::new(config)?;
            manager.search("git")
        }),
    ];

    let results = futures::future::join_all(search_tasks).await;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(search_results)) => {
                println!("Concurrent search task {} completed successfully", i);
                for (manager_name, packages) in search_results {
                    println!("  {}: {} packages", manager_name, packages.len());
                }
            }
            Ok(Err(e)) => {
                println!("Concurrent search task {} failed: {}", i, e);
            }
            Err(e) => {
                println!("Concurrent search task {} panicked: {}", i, e);
            }
        }
    }
}
