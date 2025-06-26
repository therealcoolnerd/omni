# 🌌 omni — Universal Linux Package Manager

<div align="center">

**One CLI. One System. One Omni.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-green.svg)]()
[![Status](https://img.shields.io/badge/status-production--ready-brightgreen.svg)]()

*The first truly universal Linux package manager that unifies apt, dnf, pacman, snap, flatpak, and AppImage under a single intelligent interface.*

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Documentation](#-documentation) • [Contributing](#-contributing)

</div>

---

## 🎯 **What is Omni?**

Omni solves Linux package management fragmentation by providing a **single, universal interface** for all package formats across all distributions. Whether you're on Ubuntu, Arch, Fedora, or any other Linux distro, Omni gives you the same powerful commands and features.

### **The Problem We Solve:**
- 🔀 **Fragmentation**: Different commands for different distros (apt vs dnf vs pacman)
- 🔄 **No Universal Rollback**: Most package managers can't revert system state
- 🔍 **Scattered Search**: Can't search across all package sources simultaneously  
- 🔒 **Inconsistent Security**: Different verification standards across managers
- 🧩 **Complex Dependencies**: No unified dependency resolution

### **The Omni Solution:**
- ✅ **One Interface**: Same commands work on Ubuntu, Arch, Fedora, everywhere
- ✅ **Real Snapshots**: True system rollback with database-backed state management
- ✅ **Universal Search**: Find packages across apt, snap, flatpak, AppImage simultaneously
- ✅ **Security-First**: GPG signatures + checksums + trust management built-in
- ✅ **Smart Dependencies**: Cross-platform dependency resolution and conflict detection

---

## ⚡ **Features**

<table>
<tr>
<td width="50%">

### 🎯 **Core Functionality**
- **Universal Installer** — apt, dnf, pacman, snap, flatpak, AppImage
- **Cross-Platform** — Ubuntu, Arch, Fedora, openSUSE, Debian
- **Manifest Support** — YAML-based project dependency definitions
- **Mock Mode** — Test installations safely with `--mock`

### 🔍 **Smart Discovery**
- **Unified Search** — Search all package sources with `omni search`
- **Package Information** — Detailed info with `omni info`
- **Auto-Detection** — Finds best package manager automatically
- **Intelligent Caching** — SQLite-backed for speed

</td>
<td width="50%">

### 📸 **State Management**
- **Real Snapshots** — Complete system state capture
- **Instant Rollback** — Revert to any previous state
- **Installation History** — SQLite-based operation tracking
- **Automatic Snapshots** — Created before major operations

### 🧠 **Intelligence & Security**
- **Dependency Resolution** — Smart conflict detection
- **Security Verification** — GPG + checksum validation  
- **Interactive Prompts** — Guided user interaction
- **Trust Management** — Flexible security policies

</td>
</tr>
</table>

---

## 🚀 **Installation**

### **Quick Install**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
sudo cp target/release/omni /usr/local/bin/
```

### **Prerequisites**
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Linux distribution with at least one supported package manager

### **Verify Installation**
```bash
omni --version
omni config show
```

---

## 📖 **Usage Examples**

### **Basic Operations**
```bash
# Install packages universally
omni install firefox                    # Auto-detects best package manager
omni install --box-type snap discord   # Force specific package manager
omni install code --url https://github.com/microsoft/vscode/releases/latest

# Remove packages
omni remove firefox
omni remove --box-type flatpak gimp

# Install from project manifest
omni install --from project.yaml
```

### **Search & Information**
```bash
# Search across all package sources
omni search "text editor"
omni search firefox --limit 10

# Get detailed package information  
omni info firefox
omni info --box-type apt firefox
```

### **Advanced Package Management**
```bash
# List installed packages
omni list
omni list --box-type snap --detailed

# Dependency resolution
omni resolve firefox --detailed
omni resolve --box-type apt docker

# Update management
omni update                    # Check available updates
omni update --all              # Update everything
omni update firefox            # Update specific package
omni update --refresh          # Refresh repositories first
```

### **Snapshots & History**
```bash
# Snapshot management
omni snapshot create "before-development-setup"
omni snapshot list
omni snapshot revert snapshot-id

# Installation history
omni history show --limit 20
omni history undo              # Undo last operation
```

### **Security & Verification**
```bash
# Verify package security
omni verify package.deb --checksum sha256:abc123...
omni verify app.AppImage --signature app.AppImage.sig

# Configuration
omni config show
omni config edit
omni config reset
```

### **Advanced Features**
```bash
# Resolve dependencies
omni resolve firefox

# Detailed dependency analysis
omni resolve --detailed --box-type apt firefox

# Verify package security
omni verify /path/to/package.deb --checksum sha256:abc123...

# Verify with signature
omni verify package.AppImage --signature package.AppImage.sig
```

---

## 📋 **Manifest Format**

Define project dependencies in `project.yaml`:

```yaml
project: "Development Environment"
description: "Complete setup for web development"

meta:
  distro_fallback: true

apps:
  - name: "code"
    box_type: "snap"
    
  - name: "firefox"
    box_type: "flatpak"
    
  - name: "docker"
    box_type: "apt"
    
  - name: "postman"
    box_type: "appimage"
    source: "https://dl.pstmn.io/download/latest/linux64"
```

---

## 🔧 **Configuration**

Omni stores configuration in `~/.config/omni/config.yaml`:

```yaml
general:
  auto_update: false
  parallel_installs: true
  max_parallel_jobs: 4
  confirm_installs: true
  log_level: "info"
  fallback_enabled: true

boxes:
  preferred_order:
    - "apt"
    - "dnf"
    - "pacman"
    - "flatpak"
    - "snap"
    - "appimage"
  disabled_boxes: []
  apt_options: ["-y"]
  dnf_options: ["-y"]
  pacman_options: ["--noconfirm"]

security:
  verify_signatures: true
  verify_checksums: true
  allow_untrusted: false
  check_mirrors: true
  signature_servers:
    - "keyserver.ubuntu.com"
    - "keys.openpgp.org"
    - "pgp.mit.edu"
  trusted_keys: []
  interactive_prompts: true

ui:
  show_progress: true
  use_colors: true
  compact_output: false
  gui_theme: "dark"
```

---

## 🏗️ **Supported Package Managers**

| Box Type | Description | Status |
|----------|-------------|--------|
| **apt** | Debian/Ubuntu packages | ✅ Full Support |
| **dnf** | Fedora/RHEL packages | ✅ Full Support |
| **pacman** | Arch Linux packages | ✅ Full Support |
| **snap** | Universal snap packages | ✅ Full Support |
| **flatpak** | Sandboxed applications | ✅ Full Support |
| **appimage** | Portable applications | ✅ Full Support |

---

## 🗂️ **Data Storage**

- **Configuration**: `~/.config/omni/`
- **Database**: `~/.local/share/omni/omni.db`
- **Logs**: `~/.local/share/omni/logs/`
- **Cache**: `~/.cache/omni/`
- **AppImages**: `~/.local/share/applications/appimages/`

---

## 🚀 **Why Omni is Revolutionary**

### **🆚 vs Traditional Package Managers**
| Feature | apt/dnf/pacman | Omni |
|---------|----------------|------|
| **Cross-distro** | ❌ | ✅ |
| **Universal formats** | ❌ | ✅ |
| **Real rollback** | ❌ | ✅ |
| **Unified search** | ❌ | ✅ |
| **Security verification** | ⚠️ Basic | ✅ Comprehensive |
| **Dependency resolution** | ⚠️ Limited | ✅ Cross-platform |

### **🆚 vs Other Universal Managers**
| Feature | Nix | Homebrew | Omni |
|---------|-----|----------|------|
| **Learning curve** | High | Medium | Low |
| **Existing packages** | Parallel | Parallel | Native |
| **System integration** | Isolated | Isolated | Native |
| **Rollback** | ✅ | ❌ | ✅ |
| **Security** | ✅ | ⚠️ | ✅ |

---

## 🧪 **Development & Testing**

### **Build from Source**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build
cargo test
```

### **Run Tests**
```bash
# Unit tests
cargo test

# Integration tests with mock mode
./target/debug/omni --mock install firefox
./target/debug/omni --mock search "text editor"
```

---

## 🤝 **Contributing**

We welcome contributions! Here's how to get started:

### **Quick Start**
1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Write tests for your changes
4. Ensure all tests pass: `cargo test`
5. Commit changes: `git commit -m 'Add amazing feature'`
6. Push to branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### **Areas for Contribution**
- 🐛 **Bug fixes** and error handling improvements
- 📦 **New package manager** support (zypper, emerge, etc.)
- 🔒 **Security enhancements** (more verification methods)
- 🌐 **Internationalization** (i18n support)
- 📚 **Documentation** improvements
- 🧪 **Test coverage** expansion

---

## 📜 **License**

GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)

See [LICENSE](LICENSE) file for details.

---

## 🙏 **Acknowledgments**

- **Rust Community** for the amazing ecosystem
- **Package Manager Teams** for the foundation we build upon
- **Linux Community** for making universal compatibility possible
- **Contributors** who help make Omni better

---

<div align="center">

**Built with ❤️ for the Linux community**

[⭐ Star this repo](https://github.com/therealcoolnerd/omni) • [🐛 Report Bug](https://github.com/therealcoolnerd/omni/issues) • [💡 Request Feature](https://github.com/therealcoolnerd/omni/issues) • [💬 Discussions](https://github.com/therealcoolnerd/omni/discussions)

</div>