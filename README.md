# 🌌 Omni — Universal Cross-Platform Package Manager

<div align="center">

**One CLI. Every OS. Every Package Manager.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20windows%20%7C%20macos-green.svg)]()
[![Status](https://img.shields.io/badge/status-production--ready-brightgreen.svg)]()

*The first truly universal package manager that unifies Linux, Windows, and macOS package management under a single intelligent interface.*

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Documentation](#-documentation) • [Contributing](#-contributing)

</div>

---

## 🎯 **What is Omni?**
🔥 Omni: Universal Cross‑Platform Package Manager

One CLI. Every OS. Every Package Manager.We’ve given code quality ✨Chef’s Kiss✨, served up docs that are straight 🔥, and cooked advanced features in the pot—coming soon, so stay tuned. 🚀

🎯 What is Omni?

Omni is your go‑to squad for slaying package management fragmentation. Whether you’re rockin’ Linux, Windows, or macOS, Omni’s got your back. Manage apt, dnf, pacman, snap, flatpak on Linux; winget, Chocolatey, Scoop on Windows; Homebrew & Mac App Store on macOS—all from one unified CLI.

Why Omni?

Cross‑Platform Fire: Same commands everywhere—no more context switching.

Unified Search: 🔍 Search all your package sources in one shot.

Snapshot & Rollback: Capture your setup and vibe back anytime.

Manifests: Shareable YAML recipes for project setups.

GUI Mode: A slick graphical interface for your lazy days.

🚀 Key Features (Peep the Tea)

Install / Remove / Update✅ Handles installs/removals on every platform with built‑in safety checks.

Unified Search🔍 Query every package registry at once and mark what’s already on your system.

Snapshots & Rollbacks💾 Save your package state, roll back if you mess up.

Manifests📜 Define your whole stack in YAML—share it, clone it, repeat.

Interactive Mode🤝 Prompts & menus for dependency conflicts, optional dependencies, and signature warnings.

Security First🔒 GPG signature checks, sandboxed executions, input validation—your safety net.

Performance & Benchmarks⚡ Blazing fast Rust core with async magic & built‑in benchmarks to catch regressions.

Cross‑OS GUI🖥️ omni gui for those who like point‑click‑go.

🔥 Hot Take: Everything above is locked and loaded. If it’s in this list—go ham! 🔥

💾 Installation & Setup

On Linux/macOS (via Cargo)

git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
# Binary lives in target/release/omni

On Windows (via Cargo)

git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
# Binary in target\release\omni.exe

Pro Tip: Use --mock to play in a sandbox—no real installs, all fun. 🎮

📋 Usage Examples

# Install Firefox (apt, winget, brew, whatever)—we handle it
omni install firefox

# Remove a package
omni remove vlc

# Search everything
omni search node

# Snapshot current setup
omni snapshot save my-dev-env

# Rollback to a snapshot
omni snapshot rollback my-dev-env

# Run the GUI
omni gui

Type omni --help for more deets.

🔧 Configuration

Create a ~/.config/omni/config.toml to tweak things:

[general]
allow_untrusted = false  # must confirm for unsigned packages
timeout = 600           # seconds for installs
enable_cache = true     # speed up searches

👀 What’s in the Pot (Coming Soon)

Remote Management (--ssh / Docker): Manage other machines like a boss—stirring the pot, nearly ready.

Advanced Dependency Resolver: Smarter conflict resolution on deck.

Extra Boxes: More package managers (Nix, Snapcraft, you name it).

We’re cooking these features on 🔥—stay tuned for the drop! 🎉

📝 Contributing

We stan open source collabs:

Fork it.

Create a feature branch.

Code, test, lint (cargo fmt, cargo clippy).

Open a PR—shine bright! ✨

See CONTRIBUTING.md for the full vibe.

👥 Community & Support

📣 Join our Discord for real‑time dev talk.

🐛 Found a bug? Hit us up in GitHub Issues.

🔒 Security concerns? Check out SECURITY.md and drop us a line.

Omni is built by the community, for the community. Let’s revolutionize package management—together! 💪

Stay cool, stay code‑savvy, and rock on! 🎶

Omni is the **only package manager that works everywhere** — manage apt, dnf, pacman, snap, flatpak on Linux, winget, Chocolatey, Scoop on Windows, and Homebrew, Mac App Store on macOS. All from one unified interface.

### **🌍 True Cross-Platform Package Management**
- **Windows IT Admin** → Manage Linux servers via SSH
- **macOS Developer** → Install packages in Linux containers  
- **DevOps Engineer** → Same commands across all environments
- **System Administrator** → Unified package management for mixed infrastructure

### **The Problem We Solve:**
- 🔀 **Platform Fragmentation**: Different commands for different OS (apt vs winget vs brew)
- 🔄 **No Universal Rollback**: Most package managers can't revert system state
- 🔍 **Scattered Search**: Can't search across all package sources simultaneously  
- 🔒 **Inconsistent Security**: Different verification standards across managers
- 🧩 **Complex Multi-Platform**: No single tool for heterogeneous environments

### **The Omni Solution:**
- ✅ **One Interface**: Same commands work on Linux, Windows, macOS
- ✅ **Real Snapshots**: True system rollback with database-backed state management
- ✅ **Universal Search**: Find packages across all platforms and package managers
- ✅ **Security-First**: GPG signatures + checksums + trust management built-in
- ✅ **Remote Management**: Manage Linux servers from Windows/macOS (coming soon)

---

## ⚡ **Features**

<table>
<tr>
<td width="50%">

### 🎯 **Universal Compatibility**
- **Linux** — apt, dnf, pacman, snap, flatpak, AppImage
- **Windows** — winget, Chocolatey, Scoop
- **macOS** — Homebrew, Mac App Store (mas)
- **Cross-Platform GUI** — Native desktop app for all OS

### 🔍 **Smart Discovery**
- **Unified Search** — Search all package sources with `omni search`
- **Auto-Detection** — Finds best package manager automatically
- **Intelligent Caching** — SQLite-backed for speed
- **System Detection** — Recognizes OS and available package managers

</td>
<td width="50%">

### 📸 **Advanced State Management**
- **Real Snapshots** — Complete system state capture
- **Instant Rollback** — Revert to any previous state
- **Installation History** — SQLite-based operation tracking
- **Cross-Platform Sync** — Consistent state across systems

### 🧠 **Enterprise Features**
- **Manifest Support** — YAML-based project dependency definitions
- **Mock Mode** — Test installations safely with `--mock`
- **Dependency Resolution** — Smart conflict detection
- **Security Verification** — GPG + checksum validation

</td>
</tr>
</table>

---

## 🚀 **Installation**

### **📦 Quick Install (All Platforms)**

#### **Linux**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
sudo cp target/release/omni /usr/local/bin/
```

#### **Windows**
```powershell
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
# Add target/release/omni.exe to PATH
```

#### **macOS**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
sudo cp target/release/omni /usr/local/bin/
```

### **Prerequisites**
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- At least one supported package manager on your system

### **Verify Installation**
```bash
omni --version
omni config show
```

---

## 📖 **Usage Examples**

### **🖥️ Cross-Platform Operations**
```bash
# Same commands work everywhere
omni install firefox                # Linux: snap/apt/dnf, Windows: winget, macOS: brew
omni install docker                 # Automatically chooses best package manager
omni install "Visual Studio Code"   # Handles package name variations per platform

# Platform-specific operations
omni install microsoft-teams        # Windows: winget
omni install --box-type mas xcode   # macOS: Mac App Store
omni install --box-type snap code   # Linux: snap packages
```

### **🔍 Universal Search**
```bash
# Search across all available package managers
omni search "text editor"           # Searches apt, winget, brew simultaneously
omni search firefox --limit 10      # Cross-platform Firefox packages
omni search --platform windows git  # Search only Windows package managers
```

### **📋 Project Manifests (Cross-Platform)**
```bash
# Install development environment on any OS
omni install --from project.yaml

# Example project.yaml:
# project: "Web Development Setup"
# packages:
#   - name: "git"           # Available on all platforms
#   - name: "nodejs"        # Cross-platform package
#   - name: "docker"        # Platform-appropriate version
#   - name: "code"          # VSCode via snap/winget/brew
```

### **🌐 Remote Management (Preview)**
```bash
# Manage Linux servers from Windows/macOS
omni --ssh user@server install docker     # Coming soon
omni --docker container install nodejs    # Coming soon
```

### **📸 Snapshots & History**
```bash
# Universal snapshot management
omni snapshot create "before-dev-setup"
omni snapshot list
omni snapshot revert snapshot-id
omni history undo                          # Works across platforms
```

### **🔧 GUI Application**
```bash
# Launch cross-platform GUI
omni gui
```

---

## 🏗️ **Supported Package Managers**

### **🐧 Linux**
| Package Manager | Distributions | Status |
|----------------|---------------|--------|
| **apt** | Debian, Ubuntu | ✅ Full Support |
| **dnf** | Fedora, RHEL, CentOS | ✅ Full Support |
| **pacman** | Arch Linux, Manjaro | ✅ Full Support |
| **snap** | Universal | ✅ Full Support |
| **flatpak** | Universal | ✅ Full Support |
| **appimage** | Universal | ✅ Full Support |

### **🪟 Windows**
| Package Manager | Description | Status |
|----------------|-------------|--------|
| **winget** | Windows Package Manager | ✅ Full Support |
| **chocolatey** | Community packages | ✅ Full Support |
| **scoop** | Developer tools | ✅ Full Support |

### **🍎 macOS**
| Package Manager | Description | Status |
|----------------|-------------|--------|
| **homebrew** | Community packages | ✅ Full Support |
| **mas** | Mac App Store | ✅ Full Support |

---

## 🌟 **Why Omni is Revolutionary**

### **🆚 vs Traditional Package Managers**
| Feature | apt/winget/brew | Omni |
|---------|----------------|------|
| **Cross-platform** | ❌ | ✅ |
| **Universal formats** | ❌ | ✅ |
| **Real rollback** | ❌ | ✅ |
| **Unified search** | ❌ | ✅ |
| **Remote management** | ❌ | ✅ (coming) |
| **GUI interface** | ❌ | ✅ |

### **🆚 vs Other Universal Solutions**
| Feature | Nix | Docker | Omni |
|---------|-----|--------|------|
| **Learning curve** | High | Medium | Low |
| **Native packages** | ❌ Isolated | ❌ Containerized | ✅ Native |
| **System integration** | ❌ | ❌ | ✅ |
| **Cross-platform** | ⚠️ Limited | ✅ | ✅ |
| **Existing workflow** | ❌ New | ❌ New | ✅ Compatible |

---

## 📋 **Manifest Format**

Create `project.yaml` for cross-platform project dependencies:

```yaml
project: "Full-Stack Development Environment"
description: "Works on Linux, Windows, and macOS"
version: "1.0.0"

packages:
  # Cross-platform essentials
  - name: "git"
    description: "Version control"
    
  - name: "nodejs"
    description: "JavaScript runtime"
    
  - name: "docker"
    description: "Containerization"
    
  # Platform-specific preferences
  - name: "code"
    box_type: "auto"  # snap on Linux, winget on Windows, brew on macOS
    description: "VS Code editor"
    
  # Direct URLs for universal packages
  - name: "postman"
    source: "https://dl.pstmn.io/download/latest/"
    description: "API testing tool"

scripts:
  post_install:
    - "git config --global init.defaultBranch main"
    - "echo 'Development environment ready!'"

environment:
  EDITOR: "code"
  NODE_ENV: "development"
```

---

## 🤝 **Real-World Use Cases**

### **🏢 Enterprise IT**
```bash
# Windows admin managing Linux fleet
omni --ssh-config fleet.yaml update --all
omni --parallel install docker,nodejs,git
```

### **👨‍💻 Development Teams**
```bash
# Same setup script works everywhere
omni install --from team-environment.yaml
# Developers on Windows, macOS, Linux get identical tools
```

### **☁️ DevOps & Cloud**
```bash
# Consistent package management across hybrid infrastructure
omni install kubernetes-cli        # Works on all platforms
omni snapshot create "pre-deploy"  # Universal rollback capability
```

### **🎓 Education & Training**
```bash
# Same commands in all lab environments
omni install python,nodejs,git     # Students learn one interface
omni gui                           # Visual interface for beginners
```

---

## 🧪 **Development & Testing**

### **Build from Source**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build
cargo test
```

### **Testing**
```bash
# Unit tests
cargo test

# Integration tests with mock mode (safe)
./target/debug/omni --mock install firefox
./target/debug/omni --mock search "text editor"

# Cross-platform GUI testing
./target/debug/omni gui
```

---

## 💖 **Sponsor this Project**

**Love Omni? Help us build the future of universal package management!**

<div align="center">

[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-ff69b4?logo=github&style=for-the-badge)](https://github.com/sponsors/therealcoolnerd)
[![PayPal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://paypal.me/therealcoolnerd)
[![Ko-Fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/therealcoolnerd)

</div>

### **Why Sponsor?**
- 🚀 **Accelerate Development** — More features, faster releases
- 🐛 **Priority Bug Fixes** — Sponsors get priority support  
- 🌟 **New Features** — Fund specific features you need
- 🏢 **Enterprise Support** — Custom integrations and consulting
- 🌍 **Open Source Impact** — Keep Omni free for everyone

### **Sponsorship Tiers**
- ☕ **$5/month** — Coffee Supporter (Name in README)
- 🍕 **$25/month** — Pizza Developer (Priority issue responses)
- 🚀 **$100/month** — Rocket Booster (Feature requests, direct contact)
- 🏢 **$500/month** — Enterprise Champion (Custom support, consulting)

Your sponsorship directly funds:
- Development time and infrastructure costs
- Cross-platform testing environments  
- Security audits and code reviews
- Documentation and tutorial creation

**[Become a Sponsor Today!](https://github.com/sponsors/therealcoolnerd)**

---

## 🤝 **Contributing**

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### **Priority Areas**
- 📦 **More Package Managers** (zypper, emerge, nix)
- 🌐 **Remote Management** (SSH, Docker integration)  
- 🔒 **Security Features** (enhanced verification)
- 📱 **Platform Expansion** (Android, iOS package managers)
- 🌍 **Internationalization** (multi-language support)

---

## 📜 **License**

GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)

See [LICENSE](LICENSE) file for details.

---

## 👨‍💻 **Author**

**Omni** is created and maintained by **[therealcoolnerd](https://github.com/therealcoolnerd)** 

*Making package management universal, secure, and fun!* 🚀

## 🙏 **Acknowledgments**

- **Cross-Platform Communities** for making universal compatibility possible
- **Package Manager Teams** for the foundation we build upon  
- **Rust Community** for the amazing ecosystem
- **Open Source Community** for continuous inspiration and feedback

---

<div align="center">

**Built with ❤️ and lots of ☕ by therealcoolnerd**

[⭐ Star this repo](https://github.com/therealcoolnerd/omni) • [🐛 Report Bug](https://github.com/therealcoolnerd/omni/issues) • [💡 Request Feature](https://github.com/therealcoolnerd/omni/issues) • [💬 Discussions](https://github.com/therealcoolnerd/omni/discussions)

**"Finally, one package manager for everything, everywhere."** 🌍

*Follow [@therealcoolnerd](https://github.com/therealcoolnerd) for more awesome projects!*

</div>
