<div align="center">

![Omni Logo](assets/logo.svg)

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Universal Package Manager**

*Linux · macOS · Windows — one command, every platform.*

[![Rust](https://img.shields.io/badge/Rust-1.75+-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-AGPL--3.0-000000?style=flat&logo=gnu&logoColor=white)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-000000?style=flat)]()
[![Version](https://img.shields.io/badge/Version-1.0.0%20Final-000000?style=flat)](CHANGELOG.md)

</div>

---

## The Problem

Every platform ships its own package manager. Every Linux distribution layers its own dialect on top. The result: installation scripts branch on OS, CI pipelines maintain separate configs, and the command you need changes depending on which machine you're sitting at.

Omni removes that branch entirely.

```
╔══════════════════════════════════════════════════════════════╗
║  Before Omni:                                                ║
║    sudo apt install git          # Debian / Ubuntu           ║
║    sudo dnf install git          # Fedora / RHEL             ║
║    sudo pacman -S git            # Arch Linux                ║
║    winget install Git.Git        # Windows                   ║
║    brew install git              # macOS                     ║
║                                                              ║
║  With Omni:                                                  ║
║    omni install git              # All of the above          ║
╚══════════════════════════════════════════════════════════════╝
```

---

## How It Works

Omni wraps the native package manager on the current platform. When you run `omni install git`, it detects what is available, selects the best option for the current system, validates the operation through a strict security whitelist, and delegates to the real package manager. Every install, update, and removal is recorded in a local SQLite database — giving you history, rollback, and snapshots without any cloud dependency.

---

## Supported Package Managers

```
┌───────────────┬──────────────────────────────────────────────────────┐
│ Linux         │ apt · dnf · pacman · snap · flatpak · zypper ·       │
│               │ emerge · nix · appimage                               │
├───────────────┼──────────────────────────────────────────────────────┤
│ macOS         │ homebrew · mas (Mac App Store)                        │
├───────────────┼──────────────────────────────────────────────────────┤
│ Windows       │ winget · chocolatey · scoop                           │
└───────────────┴──────────────────────────────────────────────────────┘
```

---

## Quick Start

### Build

```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
# Binary: target/release/omni
```

Optional feature flags:

```bash
cargo build --release --features gui          # Desktop GUI
cargo build --release --features ssh          # SSH remote management
cargo build --release --features gui,ssh      # Full build
```

### Core Commands

```bash
omni install <package>                # Install a package
omni remove  <package>                # Remove a package
omni search  <query>                  # Search all available managers
omni info    <package>                # Package details
omni list                             # Installed packages
omni list --detailed                  # With version and source
omni update                           # Show available updates
omni update --all                     # Apply all updates
omni update --refresh                 # Refresh repos then update
```

Specify a manager explicitly:

```bash
omni install firefox --box-type flatpak
omni remove  vlc     --box-type snap
```

---

## Feature Reference

### History and Rollback

Every operation is logged with timestamp, package name, version, and source.

```bash
omni history show                     # Full history (last 20 entries)
omni history show --limit 50          # More entries
omni history undo                     # Reverse the last operation
```

### Snapshots

Capture the exact set of installed packages at any point and restore to it.

```bash
omni snapshot create "clean-base"
omni snapshot create "pre-experiment" --description "Before testing new stack"
omni snapshot list
omni snapshot revert "clean-base"
```

### Dependency Resolution

Inspect the full resolution plan before committing to an install.

```bash
omni resolve <package>                # Resolution summary
omni resolve <package> --detailed     # Per-package dependency tree
omni resolve <package> --box-type apt # Constrain to one manager
```

### Security Verification

Verify downloaded packages before trusting them.

```bash
omni verify /path/to/pkg.deb --checksum <sha256>
omni verify /path/to/pkg.rpm --signature /path/to/pkg.rpm.sig
```

### Hardware Detection and Driver Management

Designed for mixed server environments where hardware varies by rack.

```bash
omni hardware detect                  # Show detected hardware
omni hardware install                 # Auto-install optimal drivers
omni hardware vendor dell             # Dell PowerEdge driver set
omni hardware vendor hp               # HP/HPE ProLiant driver set
omni hardware vendor supermicro       # Supermicro driver set
omni hardware vendor lenovo           # Lenovo ThinkSystem driver set
```

### Repository Management

```bash
omni repository add    <url>          # Add a package repository
omni repository remove <name>         # Remove a repository
omni repository list                  # Show configured repositories
omni repository refresh               # Refresh repository metadata
```

### Manifest Installation

Install a predefined list of packages from a YAML manifest.

```bash
omni install --from packages.yaml
```

### Web Dashboard

A local REST API server with a browser-based interface for visual package management.

```bash
omni web --port 3000
# Navigate to http://127.0.0.1:3000
```

REST endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/system/info` | OS, architecture, detected managers |
| GET | `/api/packages/installed` | All tracked installed packages |
| GET | `/api/packages/search?q=<query>` | Live package search |
| POST | `/api/packages/install` | Install a package |
| POST | `/api/packages/remove` | Remove a package |

### GUI (Optional Feature)

```bash
omni gui
# Requires: cargo build --features gui
```

### Configuration

```bash
omni config show                      # Print current config
omni config edit                      # Open config in $EDITOR
omni config reset                     # Restore defaults
```

Config file: `~/.config/omni/config.yaml` — see [CONFIGURATION.md](CONFIGURATION.md).

### Mock Mode

Simulate any operation without touching the system. Safe for CI and scripting.

```bash
omni --mock install firefox
omni --mock remove curl
omni --mock update --all
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│  CLI  —  clap-based command parser  (main.rs)                       │
├─────────────────────────────────────────────────────────────────────┤
│  OmniBrain  —  orchestration, history, rollback  (brain.rs)         │
├─────────────┬───────────────┬───────────────┬───────────────────────┤
│  Boxes      │  Resolver     │  Search       │  Security             │
│  ─────────  │  ──────────── │  ──────────── │  ─────────────────── │
│  apt        │  BFS dep      │  Cross-PM     │  GPG verification     │
│  dnf        │  graph        │  search       │  SHA-256 checksum     │
│  pacman     │  Conflict     │  Dedup &      │  Command whitelist    │
│  winget     │  detection    │  ranking      │  Argument sanitizer   │
│  brew       │  Size calc    │               │  Privilege control    │
│  + 9 more   │               │               │                       │
├─────────────┴───────────────┴───────────────┴───────────────────────┤
│  SQLite database  ·  YAML config  ·  tracing/tracing-appender logs  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Security Model

Omni executes with elevated privileges when needed. The security model reflects that responsibility.

- **Command whitelist** — every binary, every subcommand, and every flag is explicitly approved before execution. Calls that reference unlisted commands are rejected outright.
- **Argument sanitization** — shell injection characters, path traversal sequences, and command substitution are blocked at the input boundary.
- **GPG verification** — packages can be checked against upstream signatures before installation.
- **Checksum validation** — SHA-256 hashes are verified on downloaded files.
- **Privilege minimization** — elevated access is requested only for operations that require it, never stored.

Report vulnerabilities privately: see [SECURITY.md](SECURITY.md).

---

## Project Structure

```
src/
├── main.rs                CLI entry point and command dispatch
├── lib.rs                 Public library surface
├── brain.rs               Core orchestration
├── resolver.rs            Dependency resolution (BFS)
├── search.rs              Cross-platform package search
├── database.rs            SQLite operations (history, cache, snapshots)
├── snapshot.rs            Snapshot create/list/revert
├── security.rs            GPG and checksum verification
├── secure_executor.rs     Whitelist enforcement and arg sanitization
├── config.rs              YAML configuration
├── distro.rs              OS/distro detection
├── hardware.rs            Hardware detection and driver management
├── server.rs              Axum REST API
├── updater.rs             Update checking and application
├── transaction.rs         Atomic transaction support
├── advanced_resolver.rs   Extended resolution logic
└── boxes/                 Per-manager implementations
    ├── apt.rs  dnf.rs  pacman.rs  snap.rs  flatpak.rs
    ├── winget.rs  chocolatey.rs  scoop.rs
    ├── brew.rs  mas.rs
    ├── zypper.rs  emerge.rs  nix.rs  appimage.rs
    └── mod.rs
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).

## Author

**therealcoolnerd** — [github.com/therealcoolnerd](https://github.com/therealcoolnerd)

---

<div align="center">

```
╔══════════════════════════════════════════════════════════════╗
║              "One CLI to rule them all."  ⚫                 ║
╚══════════════════════════════════════════════════════════════╝
```

[Star this repo](https://github.com/therealcoolnerd/omni) · [Report a bug](https://github.com/therealcoolnerd/omni/issues) · [Read the docs](docs/user-guide.md)

</div>
