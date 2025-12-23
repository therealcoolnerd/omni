# 🌟 **Omni Universal Package Manager** — The Ultimate Cross-Platform Solution
 
 <div align="center">
 
 ![Omni Logo](assets/logo.svg)
 
 **Universal Cross-Platform Package Manager | Linux | Windows | macOS | Web Dashboard**
 
 *The only package manager you'll ever need. one command to rule them all.*
 
 [![Rust](https://img.shields.io/badge/Rust-1.75+-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)
 [![React](https://img.shields.io/badge/React-18.0+-61DAFB?style=flat&logo=react&logoColor=black)](https://react.dev)
 [![License](https://img.shields.io/badge/License-AGPL--3.0-000000?style=flat&logo=gnu&logoColor=white)](LICENSE)
 [![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-000000?style=flat&logo=linux&logoColor=white)]()
 [![GitHub](https://img.shields.io/badge/GitHub-Open%20Source-000000?style=flat&logo=github&logoColor=white)](https://github.com/therealcoolnerd/omni)


```ascii
    ╔═══════════════════════════════════════════════════════════╗
    ║  🎯 THE GOAL: One command that works everywhere           ║
    ║  ⚡ THE APPROACH: Simple, fast, reliable                  ║
    ║  🔧 THE REALITY: What actually works                      ║
    ╚═══════════════════════════════════════════════════════════╝
```

### **🔥 New in v1.0.0**
- **✨ Premium Web Dashboard**: Manage packages visually with our new React-based UI.
- **🚀 Server Mode**: New REST API backend (`src/server.rs`) using Axum.
- **🛡️ Enhanced Security**: Dependency auditing and hardened verified builds.
- **📦 Expanded Support**: Better detection for heterogeneous environments.

[⚡ **Quick Start**](#-quick-start) • [📦 **What Works**](#-what-works) • [🛠️ **Install**](#-installation) • [⚙ **Configuration**](CONFIGURATION.md)

</div>

---

## 🚀 **The Mission**

**The problem**: Every developer knows the pain. Windows has `winget`. macOS has `brew`. Ubuntu has `apt`. Arch has `pacman`. It's 2025 and we're still memorizing different commands for every platform.

**The solution**: `omni install firefox` — one command that works everywhere.

**What we built**: A single, focused package manager that wraps the native package managers on each platform. No bloat, no enterprise complexity, just working cross-platform package management.

---

## ⚡ **Quick Start**

### **🛠️ Installation**

```bash
# Build from source
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release

# The binary will be in target/release/omni
```

### **🔥 The Universal Command**
```bash
# Instead of this platform chaos:
sudo apt install firefox          # Linux
winget install Firefox           # Windows  
brew install firefox             # macOS

# Just do this everywhere:
omni install firefox            # Works on ALL platforms 🔥
```

---

## 📦 **What Works**

### **Supported Package Managers**

| Platform | Package Managers |
|----------|------------------|
| **Linux** | apt, dnf, pacman, snap, flatpak, zypper, emerge, nix |
| **macOS** | homebrew, mas (App Store) |
| **Windows** | winget, chocolatey, scoop |

### **Core Commands**

```bash
omni install <package>     # Install a package
omni remove <package>      # Remove a package
omni search <query>        # Search for packages
omni list                  # List installed packages
omni update               # Update packages
omni info <package>       # Get package information
```

### **Hardware Detection & Driver Management**

```bash
# Detect server hardware and show configuration
omni hardware detect

# Auto-detect and install optimal drivers for mixed servers
omni hardware install

# Install vendor-specific drivers (Dell, HP, Supermicro, etc.)
omni hardware vendor <vendor-name>
```

### **Optional Features**

```bash
# GUI interface (if compiled with --features gui)
omni gui

# Premium Web Dashboard (New!)
omni web --port 3000

# SSH remote management (if compiled with --features ssh)
omni ssh <host> install <package>
```

---

## 🖥️ **Mixed Server Scenarios**

Omni excels in **heterogeneous server environments** where you need to manage different hardware vendors and configurations:

### **Server Hardware Support**

| Vendor | Drivers & Tools |
|--------|----------------|
| **Dell** | dell-smbios, dcdbas, dell-wmi |
| **HP/HPE** | hpilo, hp-wmi, hp-health |
| **Supermicro** | ipmi_si, ipmi_devintf, supermicro-bmc |
| **Lenovo** | thinkpad-acpi, lenovo-wmi |
| **Cisco UCS** | cisco-ucs, cisco-enic |

### **Common Use Cases**

```bash
# Mixed datacenter with different vendors
omni hardware detect                    # Identify all server hardware
omni hardware install                   # Install optimal drivers automatically

# Specific vendor environments  
omni hardware vendor dell              # Dell PowerEdge servers
omni hardware vendor hp                # HP ProLiant servers
omni hardware vendor supermicro        # Supermicro servers

# Network-attached storage and specialized hardware
omni install mellanox-drivers          # High-speed networking
omni install nvidia-driver             # GPU compute workloads
omni install intel-ethernet            # Intel network adapters
```

### **Why This Matters**

- **Consistent tooling** across different server vendors
- **Automated driver detection** for optimal performance  
- **Mixed cloud/on-premise** deployments simplified
- **Hardware vendor independence** in your automation scripts

---

## 🏗️ **Architecture**

```ascii
┌─────────────────────────────────────────────────┐
│                 Omni CLI                        │
├─────────────────────────────────────────────────┤
│ Cross-Platform Package Manager Detection       │
├─────────────────────────────────────────────────┤
│  Linux     │   macOS      │   Windows          │
│  ────────  │   ─────────  │   ───────────      │
│  apt       │   homebrew   │   winget           │
│  dnf       │   mas        │   chocolatey       │
│  pacman    │              │   scoop            │
│  snap      │              │                    │
│  flatpak   │              │                    │
└─────────────────────────────────────────────────┘
```

---

## 🚀 **Development**

### **Build Requirements**
- Rust 1.70+
- Cargo

### **Build Commands**
```bash
# Standard build
cargo build --release

# With GUI support
cargo build --release --features gui

# With SSH support  
cargo build --release --features ssh

# Full features
cargo build --release --features gui,ssh
```

### **Project Structure**
```
src/
├── main.rs              # CLI interface
├── lib.rs               # Library entry point
├── boxes/               # Package manager implementations
│   ├── apt.rs          # Debian/Ubuntu
│   ├── brew.rs         # macOS Homebrew
│   ├── winget.rs       # Windows
│   └── ...             # Other package managers
├── config.rs           # Configuration management
├── database.rs         # SQLite operations
├── brain.rs            # Core logic
└── gui.rs              # Optional GUI (feature-gated)
```

---

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test on multiple platforms
5. Submit a pull request

### **Adding Package Manager Support**

To add a new package manager:

1. Create `src/boxes/newmanager.rs`
2. Implement the package manager interface
3. Add detection logic in `src/distro.rs`
4. Test on the target platform

---

## 📄 **License**

AGPL-3.0-or-later - see [LICENSE](LICENSE) for details.

---

## 🔥 **Built By**

**[therealcoolnerd](https://github.com/therealcoolnerd)** — *Making package management work everywhere*

📧 **Contact**: arealcoolcompany@gmail.com  
💼 **Business**: Available for consulting and custom integrations

---

<div align="center">

**[⭐ Star this repo](https://github.com/therealcoolnerd/omni)** • **[🐛 Report Issues](https://github.com/therealcoolnerd/omni/issues)** • **[💡 Request Features](https://github.com/therealcoolnerd/omni/discussions)**

---

## 🔍 **Keywords & Topics**

**Package Manager** | **Cross Platform** | **Universal Package Manager** | **Web Dashboard** | **Dashboard** | **React** | **Rust** | **Linux Package Manager** | **Windows Package Manager** | **macOS Package Manager** | **apt wrapper** | **brew wrapper** | **winget wrapper** | **dnf wrapper** | **pacman wrapper** | **snap wrapper** | **flatpak wrapper** | **Package Management Tool** | **Software Installation** | **Command Line Tool** | **CLI Package Manager** | **Rust Package Manager** | **Open Source Package Manager** | **Multi Platform** | **System Administration** | **DevOps Tools** | **Developer Tools** | **System Package Manager** | **Package Installation** | **Software Management** | **Cross Platform Development** | **Linux Tools** | **Windows Tools** | **macOS Tools** | **System Tools** | **Package Management System**

*Omni Universal Package Manager - The ultimate cross-platform package management solution for Linux, Windows, and macOS. Includes a premium web dashboard for easy management. Install software packages with one command across all operating systems. Open source Rust-based package manager that wraps apt, brew, winget, dnf, pacman, snap, flatpak and more.*

### **"One CLI to rule them all."** ⚫

*No marketing hype. Just working cross-platform package management.*

</div>