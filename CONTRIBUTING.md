<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Contributing to Omni**

</div>

---

Omni is a completed, stable project. Contributions are welcome for bug fixes, platform additions, and documentation improvements. This guide covers everything you need.

---

## Getting Started

```bash
# Fork and clone
git clone https://github.com/your-username/omni.git
cd omni

# Keep the official upstream handy
git remote add upstream https://github.com/therealcoolnerd/omni.git

# Build and confirm tests pass
cargo build
cargo test

# Work on a descriptive branch
git checkout -b fix/describe-your-change
```

Maintainers pointing origin to the official repo:

```bash
git remote set-url origin https://github.com/therealcoolnerd/omni.git
```

---

## Development Philosophy

```
╔──────────────────────────────────────────────────────────────╗
║  Security first.   Never trade user safety for convenience.  ║
║  One binary.       Same behavior on Linux, macOS, Windows.   ║
║  Minimal surface.  Do one thing and do it correctly.         ║
║  Trust nothing.    Validate at every input boundary.         ║
╚──────────────────────────────────────────────────────────────╝
```

Omni runs with elevated privileges. Every line that touches process execution, argument handling, or file paths is security-critical. Treat it that way.

---

## Project Structure

```
src/
├── main.rs              CLI entry point and command dispatch
├── brain.rs             Core orchestration
├── resolver.rs          Dependency resolution (BFS)
├── search.rs            Cross-platform search
├── database.rs          SQLite history, cache, snapshots
├── snapshot.rs          Snapshot create/list/revert
├── security.rs          GPG and checksum verification
├── secure_executor.rs   Command whitelist and arg sanitization
├── config.rs            YAML configuration
├── distro.rs            OS and package manager detection
├── hardware.rs          Hardware detection and drivers
├── server.rs            Axum REST API
├── updater.rs           Update checking and application
└── boxes/               Per-manager implementations
    ├── apt.rs  dnf.rs  pacman.rs  snap.rs  flatpak.rs
    ├── winget.rs  chocolatey.rs  scoop.rs
    ├── brew.rs  mas.rs
    ├── zypper.rs  emerge.rs  nix.rs  appimage.rs
    └── mod.rs
```

---

## Adding a New Package Manager

1. Create `src/boxes/newmanager.rs` and implement the `PackageManager` trait from `distro.rs`.
2. Add `pub mod newmanager;` to `src/boxes/mod.rs`.
3. Add detection logic in `distro.rs` → `get_available_package_managers`.
4. Add the manager's subcommands and flags to `SecureExecutor::new()` in `secure_executor.rs`. Every flag used by the implementation must be listed explicitly.
5. Re-export from `src/lib.rs` if the type is part of the public API.
6. Test on the target platform using `cargo test` and `--mock` mode.

---

## Code Standards

### Formatting and Linting

```bash
cargo fmt          # Required before every commit
cargo clippy       # All warnings must be resolved
```

The project uses strict lint settings in `Cargo.toml`. PRs with clippy warnings are not merged.

### Async Process Execution

All process execution inside `async fn` must use `tokio::process::Command`, not `std::process::Command`. Blocking the executor thread causes starvation and potential deadlocks under concurrent load.

### Adding Commands to SecureExecutor

When a package manager implementation calls `SecureExecutor::execute_package_command`, every subcommand and flag it uses must be in the `allowed_commands` map in `SecureExecutor::new()`. Do not use wildcard entries. Document any new additions in your PR description.

### Commit Message Format

```
type(scope): short description

feat(boxes): add zypper support for openSUSE
fix(executor): add missing winget flags to whitelist
docs(readme): update installation instructions
test(resolver): add conflict detection test coverage
```

Types: `feat` · `fix` · `docs` · `test` · `refactor` · `chore`

---

## Testing

```bash
# Full test suite
cargo test

# Safe integration test — nothing touches the real system
./target/debug/omni --mock install firefox
./target/debug/omni --mock remove curl
./target/debug/omni --mock update --all

# Benchmarks
cargo bench
```

See [TESTING.md](TESTING.md) for the complete testing guide.

---

## Pull Request Checklist

Before opening a PR:

- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test` passes
- [ ] Tested with `--mock` mode where applicable
- [ ] New `SecureExecutor` commands documented in the PR
- [ ] Security implications noted for privilege, process execution, or network changes
- [ ] Platforms affected are clearly stated

### Review Process

1. CI must pass (build, tests, clippy)
2. Security review for any changes to `secure_executor.rs`, `security.rs`, or privilege paths
3. Platform review for OS-specific behavior changes
4. Final approval from @therealcoolnerd

---

## Reporting Security Issues

Do **not** open public issues for security vulnerabilities.

- Use GitHub's [private vulnerability reporting](https://github.com/therealcoolnerd/omni/security/advisories/new)
- Email: **arealcoolcompany@gmail.com**

See [SECURITY.md](SECURITY.md) for the full policy and disclosure timeline.

---

## No Git Hooks Required

No hooks are enforced. Sample scripts are in [`docs/git-hook-samples/`](docs/git-hook-samples/) for optional local use.

---

<div align="center">

```
╔──────────────────────────────────────────────────────────────╗
║          Built with Rust.  Shipping on all platforms.  ⚫    ║
╚──────────────────────────────────────────────────────────────╝
```

</div>
