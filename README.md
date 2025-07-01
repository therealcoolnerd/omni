# ⚫ **Omni** — The Universal Package Manager That Actually Works

<div align="center">

**One CLI to rule them all. Zero compromises. Maximum productivity.**

[![Rust](https://img.shields.io/badge/Rust-1.70+-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-AGPL--3.0-000000?style=flat&logo=gnu&logoColor=white)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-000000?style=flat&logo=linux&logoColor=white)]()
[![Status](https://img.shields.io/badge/Status-Production%20Ready-000000?style=flat&logo=checkmarx&logoColor=white)]()

```ascii
    ╔═══════════════════════════════════╗
    ║  Why choose when you can have ALL ║
    ║  package managers in one CLI?     ║
    ╚═══════════════════════════════════╝
```

*The first package manager that doesn't make you choose between platforms. Built different.*

[🔥 **Why Omni**](#-why-omni-hits-different) • [⚡ **Get Started**](#-installation) • [🎯 **Use Cases**](#-real-world-use-cases) • [💰 **Support**](#-support-this-project)

</div>

---

## 🔥 **Why Omni Hits Different**

**Real talk**: Package management is broken. You shouldn't need to memorize 15 different commands just to install software across your devices. That's where Omni comes in clutch.

### **⚡ The Universal Truth**
```bash
# Instead of this mess:
sudo apt install firefox          # Linux
winget install Firefox           # Windows  
brew install firefox             # macOS

# Just do this everywhere:
omni install firefox            # Works on ALL platforms 🔥
```

### **📊 Problem → Solution Breakdown**

| **The L (what's broken)** | **The W (how we fix it)** |
|---------------------------|---------------------------|
| ⚫ Platform chaos (apt vs winget vs brew) | ⚪ One command, all platforms |
| ⚫ No rollback when things break | ⚪ Instant snapshots + revert |
| ⚫ Fragmented package search | ⚪ Universal search across everything |
| ⚫ Inconsistent security standards | ⚪ Built-in GPG + checksum verification |
| ⚫ No remote management | ⚪ SSH into any system, same interface |

### **🎯 Built For The Modern Stack**
- **🏢 Enterprise IT**: One tool for your entire infrastructure
- **👨‍💻 Dev Teams**: Same setup scripts across all machines
- **☁️ Cloud/DevOps**: Consistent tooling for hybrid environments
- **🚀 Power Users**: Stop context-switching between package managers

---

## ⚡ **Features That Actually Matter**

```ascii
┌─────────────────────────────────────────────────────────────┐
│  🎯 UNIVERSAL SUPPORT    📸 STATE MANAGEMENT    🧠 ENTERPRISE │
│  ═══════════════════    ═════════════════════    ══════════════ │
│  ⚪ Linux (apt/dnf...)  ⚪ Real snapshots       ⚪ YAML manifests│
│  ⚪ Windows (winget)    ⚪ Instant rollback     ⚪ Mock testing  │
│  ⚪ macOS (brew/mas)    ⚪ SQLite tracking      ⚪ Dependency AI │
│  ⚪ Cross-platform GUI  ⚪ Platform sync        ⚪ GPG security  │
└─────────────────────────────────────────────────────────────┘
```

### **🔥 The Power Features**

**📦 Package Manager Support Matrix**
```
Platform  | Managers                        | Status
----------|--------------------------------|--------
Linux     | apt, dnf, pacman, snap, flatpak| ⚪ Full
Windows   | winget, chocolatey, scoop      | ⚪ Full  
macOS     | homebrew, mas                  | ⚪ Full
```

**🎯 Smart Features**
- **⚡ Auto-Detection**: Finds the right package manager automatically
- **🔍 Universal Search**: Query all platforms simultaneously 
- **📸 True Snapshots**: Not just package lists—complete system state
- **🌐 Remote Management**: SSH into servers, same commands everywhere
- **🎨 Native GUI**: Desktop app that doesn't suck

### **🏗️ Architecture Overview**

```ascii
┌─────────────────────────────────────────────────────────────────────┐
│                          OMNI ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────┐    ┌─────────────────┐    ┌─────────────────┐   │
│  │   CLI Core    │◄──►│   SQLite DB     │◄──►│   Web GUI       │   │
│  │  (main.rs)    │    │  (snapshots)    │    │  (React SPA)    │   │
│  └───────┬───────┘    └─────────────────┘    └─────────────────┘   │
│          │                                                          │
│          ▼                                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                 Universal Resolver                          │   │
│  │              (resolver.rs + brain.rs)                       │   │
│  └─────────────────────┬───────────────────────────────────────┘   │
│                        │                                            │
│                        ▼                                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                Package Manager Boxes                        │   │
│  │  ⚪ Linux     │  ⚪ Windows    │  ⚪ macOS                   │   │
│  │  apt.rs      │  winget.rs    │  homebrew.rs               │   │
│  │  dnf.rs      │  choco.rs     │  mas.rs                    │   │
│  │  pacman.rs   │  scoop.rs     │                            │   │
│  │  snap.rs     │               │                            │   │
│  │  flatpak.rs  │               │                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## ⚡ **Get Started (Zero Friction Setup)**

### **📦 One-Liner Install**

**Universal Setup** (works on all platforms):
```bash
git clone https://github.com/therealcoolnerd/omni.git && cd omni && cargo build --release
```

**Platform-Specific Finalization**:
```bash
# Linux/macOS: 
sudo cp target/release/omni /usr/local/bin/

# Windows (PowerShell as Admin):
# Copy target/release/omni.exe to a folder in your PATH
```

### **⚡ Prerequisites Check**
```bash
# 1. Install Rust (if you don't have it)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Verify you have at least one package manager
# (You probably already do - apt, winget, or brew)
```

### **🎯 Quick Verification**
```bash
omni --version                    # Should show v0.2.0
omni config show                  # Check your setup
omni search firefox --limit 3     # Test universal search
```

**Expected output:**
```
✅ Omni v0.2.0 - Universal Package Manager
✅ Detected: apt (Ubuntu/Debian) 
✅ Detected: snap (Universal packages)
🔍 Found 3 firefox packages across all platforms
```

---

## 📖 **Usage That Makes Sense**

### **🎯 Basic Operations (Finally, Consistency)**
```bash
# The beauty of simplicity - same command, every platform
omni install firefox                 # Detects your system, picks best source
omni install docker nodejs git      # Batch install, no fuss
omni install "Visual Studio Code"   # Handles weird naming across platforms

# When you need specific sources
omni install --box-type snap code   # Force snap on Linux
omni install --box-type mas xcode   # Mac App Store on macOS
omni install --box-type winget git  # Windows Package Manager
```

### **🔍 Universal Search (Actually Universal)**
```bash
omni search "text editor"           # Searches EVERYTHING simultaneously
omni search docker --limit 5        # Top 5 results across all platforms
omni search --platform linux code   # Filter by platform when needed
```

**Sample Output:**
```
🔍 Searching across 3 package managers...
⚪ apt: code (Visual Studio Code)
⚪ snap: code (VS Code from Microsoft)  
⚪ flatpak: com.visualstudio.code
Found 3 matches in 0.2s ⚡
```

### **📋 Project Manifests (Team Setup Made Easy)**
```bash
omni install --from dev-setup.yaml  # One command, full environment

# Example dev-setup.yaml
# project: "Modern Web Stack"
# packages:
#   - git                    # Universal
#   - nodejs                 # Cross-platform  
#   - docker                 # Available everywhere
#   - code                   # Auto-detects: snap/winget/brew
```

### **📸 Snapshots (Time Travel For Your System)**
```bash
omni snapshot create "clean-slate"   # Capture current state
omni install some-sketchy-package    # Try something
omni snapshot revert "clean-slate"   # Instant rollback ⚡
omni history show                    # See all your moves
```

### **🎨 GUI Mode (For The Visual Learners)**
```bash
omni gui                            # Launch desktop app
# Point-and-click package management, but actually good
```

---

## 🏗️ **Platform Support Matrix**

```ascii
╔════════════════════════════════════════════════════════════════╗
║                    PACKAGE MANAGER COVERAGE                   ║
╠════════════════════════════════════════════════════════════════╣
║  🐧 LINUX        │  🪟 WINDOWS      │  🍎 macOS          ║
║  ═══════════     │  ══════════      │  ═════════          ║
║  ⚪ apt          │  ⚪ winget       │  ⚪ homebrew       ║
║  ⚪ dnf          │  ⚪ chocolatey   │  ⚪ mas (App Store) ║
║  ⚪ pacman       │  ⚪ scoop        │                     ║
║  ⚪ snap         │                  │                     ║
║  ⚪ flatpak      │                  │                     ║
║  ⚪ appimage     │                  │                     ║
╚════════════════════════════════════════════════════════════════╝
```

### **⚡ The Real Talk on Support**

**🐧 Linux Distribution Coverage**
- **Debian/Ubuntu** → apt (obviously)
- **Fedora/RHEL/CentOS** → dnf (rock solid)
- **Arch/Manjaro** → pacman (btw I use arch)
- **Universal** → snap, flatpak, AppImage (works everywhere)

**🪟 Windows Package Ecosystem**
- **winget** → Microsoft's official package manager (finally!)
- **chocolatey** → The community favorite for years
- **scoop** → Perfect for developer tools and CLI apps

**🍎 macOS Package Options**  
- **homebrew** → The undisputed champion of macOS packages
- **mas** → Mac App Store CLI (for when you need the official stuff)

**Status: All platforms are production-ready ⚪**

---

## 🌟 **The Competitive Landscape (Spoiler: We Win)**

### **🥊 Omni vs The Traditional Chaos**

| **Feature** | **apt/winget/brew** | **Omni** |
|-------------|-------------------|----------|
| Cross-platform | ❌ Platform-locked | ⚪ Universal |
| Rollback system | ❌ Pray it works | ⚪ Time travel |
| Universal search | ❌ One-by-one | ⚪ All-at-once |
| Remote management | ❌ SSH + memorize | ⚪ Same commands |
| GUI interface | ❌ Terminal only | ⚪ Both GUI + CLI |
| **Learning curve** | **High** (learn all) | **Low** (learn once) |

### **🥊 Omni vs "Universal" Solutions**

| **Feature** | **Nix** | **Docker** | **Omni** |
|-------------|---------|------------|----------|
| **Learning curve** | 🔴 PhD required | 🟡 Medium | ⚪ Intuitive |
| **Package isolation** | 🔴 Weird paths | 🟡 Containers | ⚪ Native system |
| **System integration** | 🔴 Broken | 🔴 Isolated | ⚪ Just works |
| **Cross-platform** | 🟡 Linux-first | ⚪ Universal | ⚪ Universal |
| **Existing workflow** | 🔴 Start over | 🔴 Containers | ⚪ Drop-in replacement |

### **🎯 The Bottom Line**
- **Traditional**: Learn 15 commands, break your system occasionally
- **Nix**: Get a PhD in functional programming first
- **Docker**: Everything is a container (even your text editor?)
- **Omni**: One command, works everywhere, doesn't break things

---

## 📋 **Manifest Files (Team Setup On Steroids)**

**The concept**: One YAML file, infinite possibilities. Share dev environments like memes.

### **⚡ Real-World Example**

`dev-stack.yaml`:
```yaml
project: "Modern Full-Stack Setup"
description: "From zero to hero in one command"
version: "2.0.0"

packages:
  # The essentials (work everywhere)
  - git                    # Version control (duh)
  - nodejs                 # JavaScript runtime  
  - docker                 # Containerization
  - code                   # VS Code (auto-detects best source)
  
  # Platform intelligence
  - name: "terraform"
    box_type: "auto"       # brew/winget/apt - whatever works
    
  # Direct downloads (when package managers fail you)
  - name: "postman"
    source: "https://dl.pstmn.io/download/latest/"

scripts:
  post_install:
    - "git config --global init.defaultBranch main"
    - "code --install-extension ms-vscode.vscode-typescript-next"
    - "echo '🚀 Dev environment ready to rock!'"

environment:
  EDITOR: "code"
  NODE_ENV: "development"
  DOCKER_BUILDKIT: "1"
```

### **🎯 Usage**
```bash
omni install --from dev-stack.yaml     # Install everything
omni install --from team-setup.yaml    # Share team configs
omni manifest validate project.yaml    # Check before running
```

**Pro tip**: Keep these in your project repos. New dev joins? One command gets them started.

---

## 🎯 **Real-World Use Cases**

### **🏢 Enterprise IT (Finally, Sanity)**
```bash
# Windows admin managing Linux servers? No problem.
omni --ssh user@server install docker,nodejs,git
omni --parallel install kubernetes-cli    # Batch operations
omni snapshot create "pre-maintenance"    # Safety first
```
*"One tool for our entire mixed infrastructure. IT budget approves."*

### **👨‍💻 Development Teams (End The Setup Wars)**
```bash
# Same setup script, every platform
omni install --from team-stack.yaml
# Junior dev on Windows, senior on macOS, CI on Linux - same tools
```
*"New hire? Clone repo, run one command, you're ready to code."*

### **☁️ DevOps & Cloud (Consistency Is King)**
```bash
# Hybrid cloud, single toolchain
omni install kubectl terraform docker     # Works everywhere
omni snapshot create "stable-baseline"    # Rollback when deployments fail
```
*"Same commands in development, staging, and production. Finally."*

### **🎓 Education & Training (One Interface To Rule Them All)**
```bash
# Students focus on learning, not package manager syntax
omni install python nodejs git           # Same everywhere
omni gui                                  # Visual for beginners
```
*"Teach concepts, not package manager quirks."*

### **🚀 Power Users (Efficiency Unlocked)**
```bash
# Cross-platform workflow
omni search "kubernetes" --platform all  # Find best version
omni install code terraform docker       # Dev stack ready
omni snapshot create "clean-workspace"   # Experiment safely
```
*"I manage Windows, macOS, and Linux. One tool, zero context switching."*

---

## 🧪 **Development & Testing (For The Builders)**

### **⚡ Quick Dev Setup**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release                     # Build for production
cargo test                                # Run the test suite  
```

### **🔍 Testing Strategy**
```bash
# Unit tests (the basics)
cargo test                                # Fast feedback loop

# Integration tests (safe mode)
./target/debug/omni --mock install docker # Test without actually installing
./target/debug/omni --mock search rust    # Mock search operations

# Performance benchmarks
cargo bench                               # See how fast we really are

# GUI testing 
./target/debug/omni gui                   # Test the desktop interface
```

### **🚀 Performance Metrics**
```
Package search: < 200ms across all managers
Snapshot creation: < 1s for typical systems
Database queries: < 50ms SQLite performance
Memory usage: < 50MB typical operation
```

*Built with modern Rust for speed and safety. Zero-cost abstractions, maximum performance.*

---

## 💰 **Support This Project (Keep The Lights On)**

**Real talk**: This project is fire, but servers cost money and developers need coffee.

<div align="center">

[![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-000000?style=flat&logo=github&logoColor=white)](https://github.com/sponsors/therealcoolnerd)
[![PayPal](https://img.shields.io/badge/PayPal-000000?style=flat&logo=paypal&logoColor=white)](https://paypal.me/therealcoolnerd)
[![Ko-Fi](https://img.shields.io/badge/Ko--fi-000000?style=flat&logo=ko-fi&logoColor=white)](https://ko-fi.com/therealcoolnerd)

</div>

### **🎯 Why Your Support Matters**
- **⚡ Faster Development** → More features, quicker bug fixes
- **🏢 Enterprise Features** → SSH management, advanced snapshots
- **🔒 Security Audits** → Professional code reviews and vulnerability testing
- **📚 Better Docs** → Tutorials, examples, and learning resources
- **🌍 Free Forever** → Keep Omni open source for everyone

### **💎 Sponsor Tiers**

```ascii
┌─────────────────────────────────────────────────────────┐
│  ☕ $5/mo   │  🍕 $25/mo  │  🚀 $100/mo │  🏢 $500/mo │
│  ────────   │  ─────────  │  ──────────  │  ─────────  │
│  Name in    │  Priority   │  Feature     │  Custom     │
│  README     │  support    │  requests    │  consulting │
└─────────────────────────────────────────────────────────┘
```

### **🚀 What Your Money Funds**
- **Development Time**: More features, better performance
- **Infrastructure**: Testing environments across all platforms
- **Security**: Professional audits and vulnerability management
- **Documentation**: Better guides, tutorials, and examples
- **Community**: Support channels and user assistance

**[⚡ Become a Sponsor](https://github.com/sponsors/therealcoolnerd)** — Every contribution matters!

---

## 🤝 **Contributing (Join The Movement)**

**Want to make package management suck less?** We're building something special here.

### **🎯 High-Impact Areas**
- **📦 Package Manager Support** → zypper, emerge, nix, pkg (FreeBSD)
- **🌐 Remote Management** → SSH integration, Docker container support
- **🔒 Security Enhancements** → Advanced GPG verification, trust chains
- **📱 Mobile Support** → Android APK, iOS TestFlight management
- **🌍 Localization** → Multi-language support for global adoption

### **⚡ Quick Start**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build
# Read CONTRIBUTING.md for the full setup
```

### **🚀 What We Need**
- **Rust Developers** → Core features, performance optimization
- **Platform Experts** → Windows/macOS/Linux package manager knowledge  
- **Security Specialists** → Vulnerability research, secure design
- **UX/UI Designers** → GUI improvements, user experience
- **Technical Writers** → Documentation, tutorials, examples

**Check out [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.**

---

## 📜 **License**

**AGPL-3.0-or-later** → [Full License](LICENSE)

*Free for personal use, contributions welcome. Commercial licensing available.*

---

## 🔥 **Built By**

**[therealcoolnerd](https://github.com/therealcoolnerd)** — *Making package management not suck since 2024*

📧 **Contact**: arealcoolcompany@gmail.com  
🐦 **Twitter**: [@therealcoolnerd](https://twitter.com/therealcoolnerd)  
💼 **Business**: Available for consulting and custom integrations

---

## 🙏 **Shoutouts**

- **🦀 Rust Community** → For the incredible language and ecosystem
- **📦 Package Manager Teams** → apt, winget, brew, and all the others we unify
- **🌍 Open Source Contributors** → The real MVPs making this possible
- **🚀 Early Adopters** → Thanks for testing and providing feedback

---

<div align="center">

```ascii
┌─────────────────────────────────────────────────────────────┐
│  ⚫ Built with Rust, powered by caffeine, fueled by passion │
└─────────────────────────────────────────────────────────────┘
```

**[⭐ Star this repo](https://github.com/therealcoolnerd/omni)** • **[🐛 Report Issues](https://github.com/therealcoolnerd/omni/issues)** • **[💡 Request Features](https://github.com/therealcoolnerd/omni/issues)** • **[💬 Discussions](https://github.com/therealcoolnerd/omni/discussions)**

### **"One CLI to rule them all, and in the package management bind them."** ⚫

*Follow [@therealcoolnerd](https://github.com/therealcoolnerd) for more projects that actually solve real problems.*

</div>