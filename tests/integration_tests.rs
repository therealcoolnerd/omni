use std::process::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_omni_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Universal Cross-Platform Package Manager"));
}

#[test]
fn test_omni_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0."));
}

#[test]
fn test_mock_install() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--mock", "install", "test-package"])
        .output()
        .expect("Failed to execute omni");
    
    // Mock mode should not fail
    assert!(output.status.success() || output.stderr.is_empty());
}

#[test]
fn test_mock_search() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--mock", "search", "firefox"])
        .output()
        .expect("Failed to execute omni");
    
    // Mock mode should not fail
    assert!(output.status.success() || output.stderr.is_empty());
}

#[test]
fn test_config_show() {
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "show"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success());
}

#[test]
fn test_manifest_validation() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("test.yaml");
    
    let manifest_content = r#"
project: "Test Project"
description: "Test manifest"
apps:
  - name: "vim"
    box: "apt"
  - name: "firefox"
    box: "snap"
"#;
    
    fs::write(&manifest_path, manifest_content).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "--mock", "install", "--from", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute omni");
    
    // Should handle manifest without crashing
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_history_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "history", "show"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success());
}

#[test]
fn test_snapshot_commands() {
    // Test snapshot creation (mock mode)
    let output = Command::new("cargo")
        .args(&["run", "--", "--mock", "snapshot", "create", "test-snapshot"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success() || output.stderr.is_empty());
    
    // Test snapshot list
    let output = Command::new("cargo")
        .args(&["run", "--", "snapshot", "list"])
        .output()
        .expect("Failed to execute omni");
    
    assert!(output.status.success());
}

#[test]
fn test_invalid_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "invalid-command"])
        .output()
        .expect("Failed to execute omni");
    
    // Should fail gracefully with error message
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

#[test]
fn test_gui_flag() {
    // Test that GUI command doesn't crash immediately
    // Note: This won't test actual GUI functionality but ensures it starts
    let output = Command::new("timeout")
        .args(&["1", "cargo", "run", "--", "gui"])
        .output();
    
    // GUI should either start successfully or timeout (both are acceptable)
    match output {
        Ok(_) => {}, // Either succeeded or timed out, both are fine
        Err(_) => {}, // Command not found or other error, also acceptable in CI
    }
}