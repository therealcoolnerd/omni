# 🔄 **Omni Migration Guide** — *Upgrade Your Package Management Game*

<div align="center">

```ascii
╔═════════════════════════════════════════════════════════════════╗
║                    🎯 MIGRATION PATHWAYS                       ║
╠═════════════════════════════════════════════════════════════════╣
║                                                                 ║
║  🚀 LITE ────→ ⚖️ CORE ────→ 🏢 ENTERPRISE                    ║
║     │              │               │                           ║
║     │              │               ↓                           ║
║     │              │         Full Features                     ║
║     │              ↓                                           ║
║     │         Power Features                                   ║
║     ↓                                                          ║
║  Daily CLI                                                     ║
║                                                                 ║
╚═════════════════════════════════════════════════════════════════╝
```

**Seamless upgrades. Backward compatibility. Zero downtime.**

</div>

---

## 🎯 **Migration Overview**

### **🔥 The Good News**
All Omni versions share:
- ✅ **Same configuration files**
- ✅ **Same package manager detection**
- ✅ **Same basic command interface**
- ✅ **Same cross-platform compatibility**
- ✅ **Backward-compatible commands**

### **⚡ Zero-Downtime Upgrades**
```bash
# Your existing workflows keep working
omni install firefox    # Works in all versions
omni search browser     # Same output format
omni list              # Same package listing
```

---

## 🚀➡️⚖️ **Lite to Core Migration**

### **🎯 Why Upgrade?**
- 📸 **Snapshots**: Backup and restore system state
- 🎯 **Manifests**: Team coordination and reproducible setups
- 🔒 **Enhanced Security**: Better package verification
- ⚙️ **Advanced Config**: More customization options

### **⚡ Migration Process (2 minutes)**

#### **Step 1: Backup Current Setup**
```bash
# Your current Lite config (if any)
cp ~/.config/omni/config.toml ~/.config/omni/config.toml.backup
```

#### **Step 2: Install Core**
```bash
# Remove Lite (optional - they can coexist)
sudo rm /usr/local/bin/omni

# Install Core
curl -sSL https://get-omni.dev/core | sh
```

#### **Step 3: Verify Migration**
```bash
omni --version    # Should show "Core Edition"
omni list         # Same packages as before
omni info         # New system information
```

#### **Step 4: Try New Features**
```bash
# Create your first snapshot
omni snapshot create "post-migration"

# Test manifest functionality
echo "packages: [git, nodejs]" > test-manifest.yaml
omni manifest validate test-manifest.yaml
```

### **🎉 Migration Complete!**
You now have all Lite features plus snapshots and manifests.

---

## ⚖️➡️🏢 **Core to Enterprise Migration**

### **🎯 Why Upgrade?**
- 🌐 **Remote Management**: SSH into servers
- 📊 **Transactions**: Atomic operations with rollback
- 🔐 **Audit Trails**: Compliance and security reporting
- 🎨 **GUI Interface**: Visual management dashboard
- 🐳 **Container Integration**: Docker/Podman support

### **⚡ Migration Process (5 minutes)**

#### **Step 1: Backup Everything**
```bash
# Backup snapshots
omni snapshot list > snapshots-backup.txt

# Backup configuration
cp -r ~/.config/omni ~/.config/omni-backup
```

#### **Step 2: Install Enterprise**
```bash
# Core and Enterprise can coexist
curl -sSL https://get-omni.dev/enterprise | sh

# Or build from source for latest features
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release --features full
sudo cp target/release/omni /usr/local/bin/omni-enterprise
```

#### **Step 3: Migration Verification**
```bash
omni --version              # Should show "Enterprise Edition"
omni snapshot list          # All snapshots preserved
omni config show           # Configuration migrated
```

#### **Step 4: Test Enterprise Features**
```bash
# Test transaction system
omni transaction begin "test-transaction"
omni install cowsay
omni transaction commit

# Test GUI (if display available)
omni gui &

# Test audit system
omni audit scan --quick
```

### **🏢 Enterprise Features Now Available**
- Remote server management
- Advanced transaction control
- Comprehensive audit trails
- GUI management interface

---

## 🔄 **Cross-Version Compatibility**

### **📋 Command Compatibility Matrix**

| **Command** | **🚀 Lite** | **⚖️ Core** | **🏢 Enterprise** |
|-------------|-------------|-------------|-------------------|
| `omni install` | ✅ Full | ✅ Full | ✅ Enhanced |
| `omni remove` | ✅ Full | ✅ Full | ✅ Enhanced |
| `omni search` | ✅ Full | ✅ Full | ✅ Enhanced |
| `omni list` | ✅ Full | ✅ Full | ✅ Enhanced |
| `omni update` | ✅ Full | ✅ Full | ✅ Enhanced |
| `omni snapshot` | ❌ N/A | ✅ Full | ✅ Enhanced |
| `omni manifest` | ❌ N/A | ✅ Full | ✅ Enhanced |
| `omni transaction` | ❌ N/A | ❌ N/A | ✅ Full |
| `omni audit` | ❌ N/A | ❌ N/A | ✅ Full |
| `omni --ssh` | ❌ N/A | ❌ N/A | ✅ Full |
| `omni gui` | ❌ N/A | ❌ N/A | ✅ Full |

### **🔧 Configuration Compatibility**

```toml
# ~/.config/omni/config.toml
# This config works across ALL versions

[general]
preferred_managers = ["apt", "brew", "winget"]
auto_update = false
confirm_before_install = true

[lite]
# Lite-specific settings (ignored by Core/Enterprise)
minimal_output = true

[core]
# Core-specific settings (ignored by Lite, used by Enterprise)
enable_snapshots = true
snapshot_retention = 30

[enterprise]
# Enterprise-specific settings (ignored by Lite/Core)
enable_audit = true
enable_ssh = true
enable_gui = true
```

---

## 🚀 **Rollback Procedures**

### **🔙 Enterprise → Core Rollback**
```bash
# Backup Enterprise data
omni audit export --all > enterprise-audit-backup.json

# Install Core
curl -sSL https://get-omni.dev/core | sh

# Verify rollback
omni --version    # Should show "Core Edition"
omni snapshot list    # Snapshots preserved
```

### **🔙 Core → Lite Rollback**
```bash
# Export snapshots (for reference)
omni snapshot list > my-snapshots.txt

# Install Lite
curl -sSL https://get-omni.dev/lite | sh

# Verify rollback
omni --version    # Should show "Lite Edition"
omni list         # Package list preserved
```

### **⚡ Emergency Rollback**
```bash
# If something goes wrong, restore from backup
cp ~/.config/omni-backup/* ~/.config/omni/

# Or use native package managers temporarily
apt install package    # Linux
brew install package   # macOS
winget install package # Windows
```

---

## 🎯 **Migration Best Practices**

### **✅ Before Migration**
1. **Document current setup**: `omni list > current-packages.txt`
2. **Backup configuration**: `cp -r ~/.config/omni ~/.config/omni-backup`
3. **Test in development**: Try new version on test machine first
4. **Plan downtime**: While minimal, plan for brief interruption

### **⚡ During Migration**
1. **Use official installers**: Avoid third-party scripts
2. **Verify each step**: Check `omni --version` after installation
3. **Test basic functionality**: `omni search test` and `omni list`
4. **Keep backups accessible**: Don't delete until verified

### **🔧 After Migration**
1. **Update team documentation**: Share new capabilities
2. **Test new features gradually**: Don't enable everything at once
3. **Monitor performance**: Ensure no regression in speed
4. **Update CI/CD**: Modify build scripts if needed

---

## 🚨 **Troubleshooting Migration Issues**

### **🔧 Common Issues & Solutions**

#### **Issue: "Command not found" after migration**
```bash
# Solution: Update PATH
export PATH="/usr/local/bin:$PATH"
# Or restart terminal
```

#### **Issue: Configuration not migrated**
```bash
# Solution: Manual migration
cp ~/.config/omni-backup/config.toml ~/.config/omni/
omni config validate
```

#### **Issue: Snapshots missing after upgrade**
```bash
# Solution: Snapshots are in Core/Enterprise only
omni snapshot list    # Should show preserved snapshots
# If empty, snapshots weren't created in previous version
```

#### **Issue: Performance regression**
```bash
# Solution: Check version and rebuild
omni --version
omni info
# If Enterprise feels slow, consider Core for your use case
```

---

## 📞 **Migration Support**

### **🆘 Need Help?**
- **Documentation**: [github.com/therealcoolnerd/omni/docs](https://github.com/therealcoolnerd/omni/docs)
- **Issues**: [github.com/therealcoolnerd/omni/issues](https://github.com/therealcoolnerd/omni/issues)
- **Discord**: [discord.gg/omni](https://discord.gg/omni)
- **Email**: support@omni.dev

### **🐛 Report Migration Bugs**
```bash
# Gather system info
omni info > migration-debug.txt
omni --version >> migration-debug.txt

# Include in GitHub issue for fastest resolution
```

---

<div align="center">

**🎉 Migration Complete! Welcome to Your New Omni Experience! 🎉**

```ascii
🚀 → ⚖️ → 🏢 = Infinite Possibilities
```

*Start simple. Scale smart. Ship fast.*

</div>