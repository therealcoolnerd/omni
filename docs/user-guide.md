# Omni Universal Package Manager - Complete User Guide

**Master Cross-Platform Package Management with Omni - The Ultimate Command Reference**

Learn how to use Omni Universal Package Manager to install, manage, and update software across Linux, Windows, and macOS with unified commands.

## 🎯 Quick Reference

### Essential Commands
```bash
# Install software
omni install firefox
omni install "visual studio code"
omni install docker nodejs python

# Search for packages
omni search browser
omni search "video editor"

# Remove software
omni remove firefox
omni remove nodejs

# Update packages
omni update
omni update firefox
omni update --all

# List installed packages
omni list
omni list --detailed

# Get package information
omni info firefox
omni info docker
```

## 📦 Package Management

### Installing Software

**Single Package Installation**
```bash
# Install from any supported package manager
omni install firefox          # Detects best package manager
omni install --from apt firefox   # Force specific manager
omni install --from brew firefox  # macOS Homebrew
omni install --from winget firefox # Windows Package Manager
```

**Multiple Package Installation**
```bash
# Install multiple packages at once
omni install git curl wget
omni install "visual studio code" discord slack

# Install from manifest file
omni install --from manifest.yml
```

**Package Manager Specific Installation**
```bash
# Linux examples
omni install --from apt firefox      # Ubuntu/Debian
omni install --from dnf firefox      # Fedora/CentOS
omni install --from pacman firefox   # Arch Linux
omni install --from snap firefox     # Ubuntu Snap
omni install --from flatpak firefox  # Flatpak

# Windows examples
omni install --from winget firefox   # Windows Package Manager
omni install --from chocolatey firefox # Chocolatey
omni install --from scoop firefox    # Scoop

# macOS examples
omni install --from brew firefox     # Homebrew
omni install --from mas firefox      # Mac App Store
```

### Searching for Software

**Basic Search**
```bash
# Search across all package managers
omni search browser
omni search "text editor"
omni search development

# Limit search results
omni search browser --limit 10
```

**Advanced Search**
```bash
# Search specific package manager
omni search --from apt browser
omni search --from brew editor

# Search with filters
omni search browser --category web
omni search editor --platform linux
```

### Package Information

**Get Detailed Package Info**
```bash
# Show package details
omni info firefox
omni info "visual studio code"

# Show dependencies
omni info firefox --dependencies

# Show available versions
omni info nodejs --versions
```

### Updating Software

**Update Single Package**
```bash
# Update specific package
omni update firefox
omni update nodejs

# Check for updates without installing
omni update firefox --check-only
```

**Update All Packages**
```bash
# Update all installed packages
omni update --all

# Update with confirmation prompts
omni update --all --interactive

# Refresh repositories first
omni update --all --refresh
```

### Removing Software

**Basic Removal**
```bash
# Remove package
omni remove firefox
omni remove nodejs

# Remove with dependencies
omni remove firefox --with-deps
```

**Advanced Removal**
```bash
# Remove and clean cache
omni remove firefox --clean

# Force removal
omni remove firefox --force

# Remove from specific manager
omni remove --from snap firefox
```

## 🔧 System Management

### Repository Management

**Add Repositories**
```bash
# Add APT repository (Linux)
omni repository add "deb http://example.com/repo stable main"
omni repository add --type ppa "ppa:user/repository"

# Add with GPG key
omni repository add "deb http://example.com/repo stable main" --key-url "http://example.com/key.gpg"
```

**List and Remove Repositories**
```bash
# List configured repositories
omni repository list

# Remove repository
omni repository remove "deb http://example.com/repo stable main"

# Refresh repository metadata
omni repository refresh
```

### Hardware and Driver Management

**Hardware Detection**
```bash
# Detect server hardware
omni hardware detect

# Auto-install recommended drivers
omni hardware install

# Install vendor-specific drivers
omni hardware vendor Dell
omni hardware vendor HP
omni hardware vendor Supermicro
```

### System Snapshots

**Create System Snapshots**
```bash
# Create snapshot before major changes
omni snapshot create "before-upgrade" --description "System state before upgrade"

# List all snapshots
omni snapshot list

# Revert to previous snapshot
omni snapshot revert "before-upgrade"
```

### Package History

**View Installation History**
```bash
# Show recent installations
omni history show

# Show last 50 installations
omni history show --limit 50

# Undo last installation
omni history undo
```

## ⚙️ Configuration

### Basic Configuration

**View Current Configuration**
```bash
# Show all configuration
omni config show

# Edit configuration file
omni config edit

# Reset to defaults
omni config reset
```

**Package Manager Settings**
```bash
# Enable/disable package managers
omni config set apt.enabled true
omni config set snap.enabled false
omni config set flatpak.enabled true

# Set preferred package manager order
omni config set priority.linux "apt,dnf,pacman,snap,flatpak"
omni config set priority.macos "brew,mas"
omni config set priority.windows "winget,chocolatey,scoop"
```

### Security Configuration

**Package Verification**
```bash
# Enable signature verification
omni config set security.verify_signatures true
omni config set security.verify_checksums true

# Verify package manually
omni verify /path/to/package.deb --checksum sha256:abcd1234...
omni verify /path/to/package.deb --signature /path/to/signature.asc
```

**Trusted Sources**
```bash
# Add trusted GPG keys
omni config add security.trusted_keys "ABCD1234..."

# Set signature servers
omni config set security.signature_servers "keyserver.ubuntu.com,keys.openpgp.org"
```

## 🚀 Advanced Features

### Dependency Resolution

**Resolve Package Dependencies**
```bash
# Show dependency tree
omni resolve firefox --detailed

# Check for conflicts
omni resolve firefox nodejs --check-conflicts

# Install with dependency resolution
omni install firefox --resolve-deps
```

### Manifest-Based Installation

**Create Installation Manifests**
```yaml
# manifest.yml
version: 1.0
name: "Development Environment"
description: "Complete development setup"

packages:
  - name: git
    manager: auto
  - name: nodejs
    manager: auto
    version: ">=18.0"
  - name: "visual studio code"
    manager: auto
  - name: docker
    manager: auto

repositories:
  - url: "ppa:deadsnakes/ppa"
    type: ppa
```

**Install from Manifest**
```bash
# Install complete environment
omni install --from manifest.yml

# Validate manifest
omni manifest validate manifest.yml

# Create manifest from current system
omni manifest generate > current-system.yml
```

### Cross-Platform Package Discovery

**Find Cross-Platform Alternatives**
```bash
# Find equivalent packages across platforms
omni discover firefox --platform all

# Get package recommendations
omni discover --category "development tools"

# Find security-audited packages
omni discover --security-verified
```

## 🐛 Troubleshooting

### Common Issues

**Package Not Found**
```bash
# Refresh repositories
omni repository refresh

# Search with broader terms
omni search "fire" # Instead of "firefox"

# Check specific package managers
omni search firefox --from apt
omni search firefox --from snap
```

**Permission Errors**
```bash
# Check privileges
omni config check-privileges

# Run with appropriate permissions
sudo omni install system-package
```

**Network Issues**
```bash
# Test connectivity
omni config test-network

# Use offline mode (if available)
omni install firefox --offline

# Configure proxy
omni config set network.proxy "http://proxy.example.com:8080"
```

### Debug Mode

**Enable Verbose Logging**
```bash
# Run with debug output
omni --verbose install firefox
omni --debug search browser

# Check logs
omni logs show
omni logs clear
```

## 📊 Performance Tips

### Speed Optimization

**Parallel Operations**
```bash
# Install multiple packages in parallel
omni install --parallel git nodejs python

# Enable caching
omni config set cache.enabled true
omni config set cache.ttl 3600  # 1 hour
```

**Efficient Updates**
```bash
# Quick update check
omni update --check-only --fast

# Background updates
omni update --all --background
```

## 🔗 Integration

### Shell Integration

**Bash/Zsh Completion**
```bash
# Add to ~/.bashrc or ~/.zshrc
eval "$(omni completion bash)"  # or 'zsh'

# Manual installation
omni completion bash > /etc/bash_completion.d/omni
```

**Aliases and Functions**
```bash
# Useful aliases
alias oi="omni install"
alias or="omni remove"
alias os="omni search"
alias ou="omni update"

# Advanced function
update-all() {
    omni repository refresh
    omni update --all
    omni clean --cache
}
```

### CI/CD Integration

**GitHub Actions**
```yaml
- name: Install dependencies with Omni
  run: |
    curl -sSL https://get.omni.sh | sh
    omni install --from .omni/manifest.yml
```

**Docker Integration**
```dockerfile
FROM ubuntu:22.04
RUN curl -sSL https://get.omni.sh | sh
COPY manifest.yml .
RUN omni install --from manifest.yml
```

## 📚 Resources

### Documentation
- **[Installation Guide](installation-guide.md)** - Setup instructions
- **[Configuration Reference](configuration.md)** - All config options
- **[Package Manager Support](package-managers.md)** - Supported systems
- **[API Reference](api-reference.md)** - Command details

### Community
- **GitHub**: [github.com/therealcoolnerd/omni](https://github.com/therealcoolnerd/omni)
- **Issues**: [Report bugs and request features](https://github.com/therealcoolnerd/omni/issues)
- **Discussions**: [Community forum](https://github.com/therealcoolnerd/omni/discussions)

---

**Keywords**: omni user guide, universal package manager commands, cross-platform software installation, package management tutorial, omni commands reference, Linux package installation, Windows software management, macOS package manager guide