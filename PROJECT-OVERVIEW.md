<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Project Overview**

</div>

---

## What Omni Is

Omni is a universal package manager written in Rust. It provides a single, consistent command-line interface across Linux, macOS, and Windows by wrapping the native package managers already present on each platform.

```
omni install git      →  apt install git          (Ubuntu/Debian)
                      →  dnf install git          (Fedora/RHEL)
                      →  pacman -S git            (Arch)
                      →  winget install Git.Git   (Windows)
                      →  brew install git         (macOS)
```

Omni detects what is available, picks the best option, validates the operation through a security whitelist, and delegates. Every operation is recorded in a local SQLite database for history, rollback, and snapshots.

---

## Status: v1.0.0 Final

This is a completed, stable release. All known bugs have been fixed. The codebase compiles cleanly on Linux, macOS, and Windows.

---

## Feature Surface

```
┌─────────────────────────────────────────────────────────────────┐
│  Core operations    install · remove · search · info · list     │
│  Update management  check · update-all · refresh repos          │
│  History            full log · undo last operation              │
│  Snapshots          create · list · revert                      │
│  Dependency graph   BFS resolution · conflict detection         │
│  Security           GPG verify · SHA-256 checksum · whitelist   │
│  Hardware           detect · auto-drivers · vendor-specific     │
│  Repository         add · remove · list · refresh               │
│  Web API            Axum REST server (localhost-only)           │
│  GUI                Desktop interface (optional feature)        │
│  SSH                Remote management (optional feature)        │
│  Mock mode          Safe testing without system changes         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Package Manager Coverage

| Platform | Managers |
|----------|----------|
| Linux | apt · dnf · pacman · snap · flatpak · zypper · emerge · nix · appimage |
| macOS | homebrew · mas |
| Windows | winget · chocolatey · scoop |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  CLI  (main.rs / clap)                                          │
├─────────────────────────────────────────────────────────────────┤
│  OmniBrain  — orchestration, history, snapshots                 │
├─────────────┬───────────────┬───────────────┬───────────────────┤
│  Boxes      │  Resolver     │  Search       │  Security         │
│  ─────────  │  ──────────── │  ──────────── │  ─────────────── │
│  Per-PM     │  BFS dep      │  Multi-PM     │  GPG / SHA-256   │
│  wrappers   │  graph        │  search +     │  Command          │
│  (14 total) │  Conflict     │  dedup        │  whitelist        │
│             │  detection    │               │  Arg sanitizer    │
├─────────────┴───────────────┴───────────────┴───────────────────┤
│  SQLite (history, cache, snapshots)                             │
│  YAML config · tracing logs · Axum REST API                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Security Design

Omni runs with elevated privileges. The security design reflects that:

- **Command whitelist** — every binary, subcommand, and flag is explicitly approved before any process is spawned
- **Argument sanitization** — shell injection, path traversal, and command substitution are blocked at the input boundary
- **Localhost-only web server** — the REST API binds on `127.0.0.1`, never exposed to the network
- **Privilege minimization** — sudo/elevation is requested only when required, never stored

---

## Technical Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2021 edition |
| Async runtime | Tokio (multi-thread) |
| CLI | clap 4.x |
| Database | SQLite via sqlx 0.8 |
| Web server | Axum 0.7 |
| Logging | tracing + tracing-appender |
| Config | serde + serde_yaml |
| GUI (optional) | eframe / egui 0.31 |
| SSH (optional) | russh 0.53 |

---

## Code Organization

```
src/
├── main.rs                CLI entry point
├── lib.rs                 Public library API
├── brain.rs               Core orchestration
├── resolver.rs            Dependency resolution
├── advanced_resolver.rs   Extended resolution logic
├── search.rs              Cross-platform search
├── database.rs            SQLite operations
├── snapshot.rs            Snapshot management
├── security.rs            GPG and checksum verification
├── secure_executor.rs     Command whitelist + sanitization
├── config.rs              YAML configuration
├── distro.rs              OS/distro detection
├── hardware.rs            Hardware detection and drivers
├── server.rs              Axum REST API
├── updater.rs             Update management
├── transaction.rs         Atomic operations
├── unified_manager.rs     Unified manager abstraction
├── audit.rs               Security audit trail
├── privilege_manager.rs   Privilege escalation control
├── sandboxing.rs          Process sandboxing
├── input_validation.rs    Input validation utilities
├── logging.rs             Logging configuration
├── manifest.rs            Manifest file parsing
├── package_discovery.rs   Package discovery service
├── interactive.rs         Interactive prompts
├── history.rs             History management
├── docker.rs              Container support
├── ssh.rs                 SSH remote management (feature)
├── gui.rs                 Desktop GUI (feature)
└── boxes/                 Per-manager implementations (14)
```

---

## Design Principles

- **Wrap, don't replace.** The native package managers are already tested and maintained. Omni translates commands; the PM does the work.
- **One path, every platform.** A script that installs with Omni should work unchanged on any supported OS.
- **Security is not optional.** Elevated privilege demands strict input handling. Every execution path goes through the whitelist.
- **Local-first.** History, snapshots, and cache live in a local SQLite database. No cloud dependency, no account required.
- **Observable.** Every operation is logged. Nothing happens silently.

---

## License and Author

- **License**: AGPL-3.0-or-later
- **Author**: [therealcoolnerd](https://github.com/therealcoolnerd)
- **Contact**: arealcoolcompany@gmail.com

---

[README](README.md) · [User Guide](docs/user-guide.md) · [Installation](docs/installation-guide.md) · [Contributing](CONTRIBUTING.md)
