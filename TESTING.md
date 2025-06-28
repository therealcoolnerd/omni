# Testing Guide for Omni

This guide covers how to run, write, and contribute tests for the Omni Universal Package Manager.

## 🚀 Quick Start

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test integration_tests

# Run benchmarks
cargo bench
```

## 📋 Test Categories

### Unit Tests (`tests/unit_tests.rs`)
Test individual components in isolation:

```bash
# Run all unit tests
cargo test unit_tests

# Run specific module tests
cargo test config_tests
cargo test manifest_tests
cargo test security_tests
```

**What to test:**
- Configuration parsing and validation
- Manifest file handling
- Database operations
- Security verification logic
- Dependency resolution algorithms

### Integration Tests (`tests/integration_tests.rs`)
Test complete workflows and CLI interactions:

```bash
# Run all integration tests
cargo test integration_tests

# Run with mock mode (safe for CI)
cargo test test_mock_install
cargo test test_mock_search
```

**What to test:**
- CLI command execution
- End-to-end package operations
- Configuration file handling
- Error scenarios and edge cases

### Performance Benchmarks (`benches/performance.rs`)
Measure and track performance:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench benchmark_brain_creation

# Generate HTML reports
cargo bench -- --output-format html
```

**What to benchmark:**
- Core component initialization
- Search operations
- Manifest parsing
- Database queries
- Dependency resolution

## 🧪 Testing Modes

### Mock Mode Testing
Use `--mock` flag for safe testing without actual package operations:

```bash
# Test installation without actually installing
./target/debug/omni --mock install firefox

# Test search functionality
./target/debug/omni --mock search "text editor"

# Test manifest installation
./target/debug/omni --mock install --from test_manifest.yaml
```

### Real Environment Testing
**⚠️ Use with caution in development environments:**

```bash
# Test with real package managers (requires sudo)
sudo ./target/debug/omni install vim

# Test GUI (requires X11/Wayland)
./target/debug/omni gui
```

### Container Testing
Use Docker for isolated testing:

```bash
# Ubuntu testing
docker run -it ubuntu:22.04 bash
# Install Rust and test Omni

# Fedora testing  
docker run -it fedora:38 bash
# Install Rust and test Omni

# Arch testing
docker run -it archlinux:latest bash
# Install Rust and test Omni
```

## 📝 Writing Tests

### Unit Test Example

```rust
#[cfg(test)]
mod my_feature_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_feature_functionality() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config = MyConfig::new();
        
        // Act
        let result = my_feature(&config, temp_dir.path());
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, expected_value);
    }
    
    #[test]
    fn test_error_handling() {
        let invalid_input = "";
        let result = my_feature_with_validation(invalid_input);
        assert!(result.is_err());
    }
}
```

### Integration Test Example

```rust
#[test]
fn test_cli_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "command", "args"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expected output"));
}
```

### Benchmark Example

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            black_box(my_function(black_box(input_data)))
        })
    });
}

criterion_group!(benches, benchmark_my_function);
criterion_main!(benches);
```

## 🔧 Test Environment Setup

### Development Environment

```bash
# Install test dependencies
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit      # Security auditing
cargo install cargo-deny       # License and security checking

# Run with coverage
cargo tarpaulin --out Html

# Security audit
cargo audit

# License and dependency checking
cargo deny check
```

### CI/CD Environment

Our GitHub Actions automatically run:
- Unit and integration tests
- Security audits
- Code coverage analysis
- Cross-platform builds
- Performance regression detection

### Local Testing Checklist

Before submitting a PR:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run all tests
cargo test

# Run benchmarks (dry run)
cargo bench --no-run

# Check security
cargo audit

# Test mock mode
./target/debug/omni --mock install test-package
./target/debug/omni --mock search firefox

# Test GUI (if modified)
./target/debug/omni gui
```

## 🏗️ Test Data and Fixtures

### Test Manifests (`tests/data/`)
- `test_manifest.yaml` - Basic test manifest
- `complex_manifest.yaml` - Multi-package manifest
- `invalid_manifest.yaml` - Invalid YAML for error testing

### Test Utilities

```rust
// Helper functions for testing
mod test_helpers {
    pub fn create_temp_config() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        (temp_dir, config_path)
    }
    
    pub fn mock_package_database() -> Database {
        Database::in_memory().unwrap()
    }
    
    pub fn skip_if_no_package_manager(pm: &str) -> bool {
        Command::new(pm).arg("--version").output().is_err()
    }
}
```

## 🐛 Testing Best Practices

### 1. Test Isolation
- Use temporary directories for file operations
- Reset global state between tests
- Mock external dependencies

### 2. Error Testing
- Test both success and failure paths
- Validate error messages and types
- Test edge cases and boundary conditions

### 3. Performance Testing
- Benchmark critical paths
- Set performance regression thresholds
- Test with realistic data sizes

### 4. Security Testing
- Test privilege escalation scenarios
- Validate input sanitization
- Test cryptographic verification

### 5. Cross-Platform Testing
- Test on multiple Linux distributions
- Verify Windows and macOS compatibility
- Test different package manager combinations

## 📊 Test Coverage

Monitor test coverage to ensure comprehensive testing:

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage

# View coverage report
open target/coverage/tarpaulin-report.html
```

**Coverage Goals:**
- Overall: >80%
- Critical modules (security, database): >95%
- GUI modules: >60% (due to testing complexity)

## 🚨 Testing Security Features

### GPG Verification Testing
```bash
# Test with valid signatures
./target/debug/omni --mock install signed-package

# Test with invalid signatures (should fail)
./target/debug/omni --mock install unsigned-package
```

### Privilege Testing
```bash
# Test operations that require sudo
sudo ./target/debug/omni install system-package

# Test user-level operations
./target/debug/omni search user-package
```

## 📚 Continuous Integration

Our CI pipeline runs:

1. **Fast Tests** (every commit)
   - Unit tests
   - Linting and formatting
   - Basic integration tests

2. **Full Test Suite** (PRs to main)
   - All integration tests
   - Cross-platform testing
   - Performance benchmarks
   - Security audits

3. **Release Testing** (tags)
   - Full test suite on all platforms
   - Performance regression tests
   - Security vulnerability scans

## 🤝 Contributing Test Improvements

When contributing tests:

1. **Write tests first** (TDD approach)
2. **Test both happy and error paths**
3. **Use descriptive test names**
4. **Include performance tests for new features**
5. **Add integration tests for CLI changes**
6. **Update this documentation** for new testing patterns

## 📞 Getting Help

- **Test Failures**: Check CI logs and run locally
- **Performance Issues**: Use `cargo bench` to identify bottlenecks
- **Security Concerns**: Run `cargo audit` and review security tests
- **Coverage Questions**: Generate coverage reports with `cargo tarpaulin`

---

Remember: Good tests make good software! 🚀