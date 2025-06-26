# 🌌 omni — Universal Linux Installer Engine

**One CLI. One System. One Omni.**

Omni is the future of Linux package installation — built to unify `.deb`, `.rpm`, Flatpak, AppImage, Snap, and beyond — under a single command-line interface that feels like magic.

With modular **Omni Boxes**, a secure **Omni Brain**, real **snapshots**, and intelligent **search**, it's designed to be your *forever installer* — whether you're on Ubuntu, Arch, Fedora, or something off the grid.

## ⚡️ Features

### 🎯 **Core Functionality**
- **Universal Installer** — Install from apt, dnf, pacman, snap, flatpak, and AppImage
- **Cross-Platform** — Works on Ubuntu, Arch, Fedora, openSUSE, and more
- **Manifest Support** — Define project dependencies in YAML/JSON
- **Mock Mode** — Test installations safely with `--mock`

### 🔍 **Smart Package Discovery**
- **Unified Search** — Search across all package sources with `omni search`
- **Package Information** — Get detailed info with `omni info`
- **Auto-Detection** — Automatically finds the best package manager
- **Caching** — Intelligent caching for faster subsequent searches

### 📸 **State Management**
- **Real Snapshots** — Create system snapshots before major changes
- **Rollback System** — Revert to any previous state instantly
- **Installation History** — Track all package operations with SQLite
- **Automatic Snapshots** — Auto-created before installs/removals

### 🔄 **Update Management**
- **Update Checking** — See available updates across all packages
- **Selective Updates** — Update specific packages or everything
- **Repository Refresh** — Keep package databases current
- **Progress Indicators** — Visual feedback for all operations

### ⚙️ **Advanced Features**
- **Configuration System** — Customize behavior via YAML config
- **Structured Logging** — Comprehensive logging to files and console
- **Async Operations** — Fast, non-blocking package operations
- **GUI Support** — Optional graphical interface

### 🧠 **Intelligence & Security**
- **Dependency Resolution** — Smart dependency detection and conflict resolution
- **Security Verification** — GPG signature and checksum verification
- **Interactive Prompts** — Smart user interaction and error recovery
- **Trust Management** — Flexible security policies and trusted key management

## 🚀 Installation

```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
sudo cp target/release/omni /usr/local/bin/
```

## 📖 Usage Examples

### 🔧 **Basic Operations**
```bash
# Install a package
omni install firefox

# Install from specific package manager
omni install --box-type snap discord

# Install AppImage from URL
omni install code --url https://github.com/microsoft/vscode/releases/download/1.84.2/code-1.84.2-x64.AppImage

# Remove a package
omni remove firefox

# Install from manifest
omni install --from project.yaml
```

### 🔍 **Search & Information**
```bash
# Search for packages
omni search "text editor"

# Get package information
omni info firefox

# Get info from specific source
omni info firefox --box-type apt
```

### 📦 **Package Management**
```bash
# List installed packages
omni list

# List with details
omni list --detailed

# Check for updates
omni update

# Update all packages
omni update --all

# Update specific package
omni update firefox

# Refresh repositories
omni update --refresh
```

### 📸 **Snapshots & History**
```bash
# Create a snapshot
omni snapshot create "before-development-setup"

# List snapshots
omni snapshot list

# Revert to snapshot
omni snapshot revert snapshot-id

# Show installation history
omni history show

# Undo last operation
omni history undo
```

### ⚙️ **Configuration**
```bash
# Show current configuration
omni config show

# Edit configuration
omni config edit

# Reset to defaults
omni config reset

# Launch GUI
omni gui
```

### 🧠 **Advanced Features**
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

## 📋 **Manifest Format**

Create `project.yaml` to define your project's dependencies:

```yaml
project: "My Development Environment"
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
```

## 🏗️ **Supported Package Managers**

| Box Type | Description | Status |
|----------|-------------|--------|
| **apt** | Debian/Ubuntu packages | ✅ Full Support |
| **dnf** | Fedora/RHEL packages | ✅ Full Support |
| **pacman** | Arch Linux packages | ✅ Full Support |
| **snap** | Universal snap packages | ✅ Full Support |
| **flatpak** | Sandboxed applications | ✅ Full Support |
| **appimage** | Portable applications | ✅ Full Support |

## 🗂️ **Data Storage**

- **Configuration**: `~/.config/omni/`
- **Database**: `~/.local/share/omni/omni.db`
- **Logs**: `~/.local/share/omni/logs/`
- **Cache**: `~/.cache/omni/`
- **AppImages**: `~/.local/share/applications/appimages/`

## 🤝 **Contributing**

We welcome contributions! Here's how to get started:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Commit: `git commit -m 'Add amazing feature'`
5. Push: `git push origin feature/amazing-feature`
6. Open a Pull Request

## 📜 **License**

GNU AGPLv3 - see [LICENSE](LICENSE) for details.

---

**Built with ❤️ for the Linux community**