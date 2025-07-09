# Omni Universal Package Manager - Complete Installation Guide

**The Ultimate Cross-Platform Package Manager Installation Instructions for Linux, Windows, macOS**

This comprehensive guide covers installing Omni Universal Package Manager on all supported operating systems. Omni is the modern solution for cross-platform package management.

## 📦 What is Omni Universal Package Manager?

Omni is an open-source universal package manager written in Rust that provides a unified interface for installing software across Linux, Windows, and macOS. Instead of learning different commands for apt, brew, winget, dnf, pacman, snap, and flatpak, you use one simple command: `omni install [package]`.

## 🚀 Quick Installation

### Linux Installation (Ubuntu, Debian, Fedora, Arch, SUSE)

**Method 1: Download Pre-built Binary**
```bash
# Download latest release
wget https://github.com/therealcoolnerd/omni/releases/latest/download/omni-linux-amd64.tar.gz

# Extract and install
tar -xzf omni-linux-amd64.tar.gz
sudo mv omni /usr/local/bin/
sudo chmod +x /usr/local/bin/omni

# Verify installation
omni --version
```

**Method 2: Build from Source**
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build Omni
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release

# Install binary
sudo cp target/release/omni /usr/local/bin/
```

**Method 3: Package Manager Installation**
```bash
# Ubuntu/Debian (coming soon)
# sudo apt install omni

# Fedora/CentOS (coming soon)  
# sudo dnf install omni

# Arch Linux (AUR - coming soon)
# yay -S omni
```

### Windows Installation

**Method 1: Windows Package Manager (WinGet)**
```powershell
# Install via WinGet (coming soon)
# winget install omni
```

**Method 2: Download Executable**
```powershell
# Download from GitHub releases
# Extract omni.exe to C:\Program Files\Omni\
# Add to PATH environment variable
```

**Method 3: Build from Source**
```powershell
# Install Rust
# Download from https://rustup.rs/
# Or use chocolatey: choco install rust

# Clone and build
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
```

### macOS Installation

**Method 1: Homebrew (Recommended)**
```bash
# Add Omni tap (coming soon)
# brew tap therealcoolnerd/omni
# brew install omni
```

**Method 2: Download Binary**
```bash
# Download for Intel Macs
wget https://github.com/therealcoolnerd/omni/releases/latest/download/omni-macos-amd64.tar.gz

# Download for Apple Silicon Macs
wget https://github.com/therealcoolnerd/omni/releases/latest/download/omni-macos-arm64.tar.gz

# Extract and install
tar -xzf omni-macos-*.tar.gz
sudo mv omni /usr/local/bin/
```

**Method 3: Build from Source**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release

# Install
sudo cp target/release/omni /usr/local/bin/
```

## ⚙️ Configuration

### First Run Setup
```bash
# Initialize Omni configuration
omni config init

# Verify package managers are detected
omni config show

# Test installation
omni search firefox
omni install firefox
```

### Advanced Configuration
```bash
# Edit configuration file
omni config edit

# Add custom repositories
omni repository add "deb http://example.com/repo stable main"

# Enable/disable specific package managers
omni config set apt.enabled true
omni config set snap.enabled false
```

## 🔧 System Requirements

### Linux Requirements
- **Supported Distributions**: Ubuntu 18.04+, Debian 10+, Fedora 32+, CentOS 8+, Arch Linux, openSUSE 15+
- **Architecture**: x86_64, ARM64
- **Dependencies**: glibc 2.28+, OpenSSL 1.1.1+
- **Package Managers**: At least one of: apt, dnf, pacman, zypper, snap, flatpak

### Windows Requirements
- **Windows Version**: Windows 10 version 1809+ or Windows 11
- **Architecture**: x86_64, ARM64
- **Dependencies**: Microsoft Visual C++ Redistributable 2019+
- **Package Managers**: winget, chocolatey, or scoop

### macOS Requirements
- **macOS Version**: 10.15 Catalina or later
- **Architecture**: Intel x86_64, Apple Silicon (M1/M2)
- **Package Managers**: Homebrew recommended

## 🚨 Troubleshooting

### Common Installation Issues

**"Command not found: omni"**
```bash
# Check if omni is in PATH
echo $PATH | grep -o "/usr/local/bin"

# Add to PATH if missing
echo 'export PATH=$PATH:/usr/local/bin' >> ~/.bashrc
source ~/.bashrc
```

**Permission Denied Errors**
```bash
# Fix permissions on Linux/macOS
sudo chown $USER:$USER /usr/local/bin/omni
sudo chmod +x /usr/local/bin/omni
```

**SSL/TLS Errors**
```bash
# Update certificates on Linux
sudo apt update && sudo apt install ca-certificates
sudo update-ca-certificates

# On CentOS/RHEL
sudo yum update ca-certificates
```

### Platform-Specific Issues

**Linux: Package Manager Not Detected**
```bash
# Install missing package managers
sudo apt install snapd flatpak  # Ubuntu/Debian
sudo dnf install snapd flatpak  # Fedora
```

**Windows: Execution Policy Errors**
```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**macOS: "Developer Cannot be Verified"**
```bash
# Allow unsigned binary (if building from source)
sudo spctl --master-disable
# Run omni once, then re-enable:
sudo spctl --master-enable
```

## 🔄 Updating Omni

### Auto-Update
```bash
# Check for updates
omni update --check

# Update Omni itself
omni update --self
```

### Manual Update
```bash
# Re-download and replace binary
wget https://github.com/therealcoolnerd/omni/releases/latest/download/omni-linux-amd64.tar.gz
tar -xzf omni-linux-amd64.tar.gz
sudo mv omni /usr/local/bin/
```

## 🔗 Next Steps

After installation:

1. **[Read the User Guide](user-guide.md)** - Learn all Omni commands
2. **[Package Manager Support](package-managers.md)** - Understand supported systems
3. **[Configuration Reference](configuration.md)** - Advanced setup options
4. **[Troubleshooting Guide](troubleshooting.md)** - Fix common issues

## 📞 Support

- **Documentation**: [GitHub Wiki](https://github.com/therealcoolnerd/omni/wiki)
- **Issues**: [GitHub Issues](https://github.com/therealcoolnerd/omni/issues)
- **Discussions**: [GitHub Discussions](https://github.com/therealcoolnerd/omni/discussions)
- **Email**: arealcoolcompany@gmail.com

---

**Keywords**: universal package manager installation, cross-platform package manager setup, Linux package manager, Windows package manager, macOS package manager, omni installation guide, apt wrapper, brew wrapper, winget wrapper, package management tool