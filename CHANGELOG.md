# Omni Changelog

All notable changes are documented here. Omni follows [Semantic Versioning](https://semver.org).

---

## [1.0.0] — 2026-03-07  ·  Final Release

This is the stable, complete release of Omni. The codebase has been audited, all known bugs fixed, and all documentation brought up to date.

### Bug Fixes

- **`secure_executor.rs`** — Removed unused `std::os::unix::process::CommandExt` import that caused compilation failure on Windows and macOS. The project compiles cleanly on all three target platforms.
- **`secure_executor.rs`** — Rewrote command validation logic. Previously only flag arguments (those starting with `-`) were checked against the whitelist; subcommands like `install` and `remove` were silently passed through unvalidated. The validator now checks the first argument (subcommand or primary flag) and all subsequent flags independently.
- **`secure_executor.rs`** — Added Windows package managers (`winget`, `choco`, `scoop`) and macOS package managers (`brew`, `mas`) to the command whitelist. Previously any call through `SecureExecutor` on non-Linux platforms was rejected with "Command not in allowed list."
- **`secure_executor.rs`** — Added missing flags actually used by the codebase: `-y` for apt/dnf/flatpak, `--names-only` and `--upgradable` for apt, `--quiet` and `--requires` for dnf, `--show-runtime` for flatpak, and `dpkg-query` with its `--showformat` flag.
- **`database.rs`** — Fixed first-run failure on fresh installations. The SQLite connection now uses `SqliteConnectOptions::create_if_missing(true)`, which creates the database file if it does not yet exist. Previously the file was never created and the process exited with a connection error on first use.
- **`server.rs`** — Removed hardcoded mock data from `GET /api/packages/installed`. The handler now queries the SQLite database directly and returns the real installed package list.
- **`server.rs`** — Wired `GET /api/packages/search` to the real `SearchEngine`. Previously the handler locked the brain mutex and then discarded it, returning a trivial one-entry mock regardless of the query.
- **`server.rs`** — Changed the listen address from `0.0.0.0` to `127.0.0.1`. The web server is a local management interface; binding on all interfaces with `CorsLayer::permissive()` exposed install and remove endpoints to any origin on the local network.
- **`resolver.rs`** — Replaced blocking `std::process::Command` with `tokio::process::Command` in all async functions (`get_apt_dependencies`, `get_dnf_dependencies`, `get_pacman_dependencies`, `get_flatpak_dependencies`, and all version/size helpers). Synchronous process spawning inside an async context blocks the executor thread and can cause deadlocks under concurrent load.
- **`search.rs`** — Same fix as resolver.rs: all `std::process::Command` calls in async search functions replaced with `tokio::process::Command`.
- **`updater.rs`** — Same fix: all update check functions now use non-blocking async process execution.
- **`resolver.rs`** — `select_best_box` now returns a platform-appropriate priority order. Previously the list was hardcoded to Linux package managers regardless of OS; on Windows and macOS the function fell back to `detect_distro()` which returns the wrong manager. The resolver now detects the current platform and selects from `[winget, choco, scoop]` on Windows or `[brew, mas]` on macOS.
- **`distro.rs`** — Fixed Windows version detection. `ver` is a `cmd.exe` shell builtin, not a standalone binary. `Command::new("ver")` always fails silently on Windows. Changed to `Command::new("cmd").args(["/C", "ver"])`.
- **`distro.rs`** — Replaced the trivially true test assertion `assert!(managers.len() >= 0)` (a `usize` is always `>= 0`) with a meaningful check that validates each returned manager name is non-empty.
- **`Cargo.toml`** — Removed the duplicated `# Network and HTTP` comment on line 67.

### Improvements

- **`server.rs`** — Added `available_managers` field to `GET /api/system/info` response, showing which package managers are detected on the current host.
- **`server.rs`** — Search endpoint now accepts an optional `limit` query parameter (default 20) to control result set size.
- **`server.rs`** — API responses are now wrapped in a consistent `{ "status": "success"|"error", "data": ..., "message": ... }` envelope.

---

## [0.2.0] — 2025-08-15

### Added

- Windows `winget` support.
- Basic SSH remote management.
- SQLite database integration for install history tracking.
- Snapshot create, list, and revert.

---

## [0.1.0] — 2025-01-10

### Added

- Initial release.
- Support for `apt`, `brew`, and `pacman`.
- CLI structure using `clap`.
- Basic dependency resolution.
