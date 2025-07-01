# ⚡ **Omni Quick Start Guide** — *Get Running in 60 Seconds*

<div align="center">

```ascii
┌─────────────────────────────────────────────────────────────┐
│  🚀 LITE      ⚖️ CORE        🏢 ENTERPRISE                 │
│  ────────     ──────────     ──────────────                │
│  Daily CLI    Power User     Mission Critical              │
│  18s build    45s build     120s build                     │
│  865KB        ~10MB         ~50MB                          │
└─────────────────────────────────────────────────────────────┘
```

**Choose your speed. Choose your power. Choose your Omni.**

</div>

---

## 🎯 **30-Second Decision Tree**

### **🤔 Which Version Do I Need?**

```bash
# ❓ Do you just want `apt install` to work on Mac/Windows?
→ 🚀 OMNI LITE (you're 90% of users)

# ❓ Do you need snapshots, manifests, and team coordination?
→ ⚖️ OMNI CORE (perfect balance)

# ❓ Do you manage infrastructure, need SSH, transactions, audits?
→ 🏢 OMNI ENTERPRISE (full power)
```

---

## 🚀 **OMNI LITE** — *For Speed Demons*

### **⚡ Install in 10 Seconds**
```bash
curl -sSL https://get-omni.dev/lite | sh
```

### **🎯 Basic Commands**
```bash
omni install firefox        # Install packages
omni remove firefox         # Remove packages  
omni search browser         # Search packages
omni list                   # List installed
omni update                 # Update cache
```

### **✨ That's It!**
You now have universal package management. No complexity. Just speed.

---

## ⚖️ **OMNI CORE** — *For Power Users*

### **⚡ Install in 30 Seconds**
```bash
curl -sSL https://get-omni.dev/core | sh
```

### **🎯 Power User Workflow**
```bash
# Basic package management (same as Lite)
omni install git nodejs docker

# Snapshot workflow (your new superpower)
omni snapshot create "clean-state"
omni install experimental-package
omni snapshot restore "clean-state"    # Instant rollback ⚡

# Team coordination
omni manifest install team-setup.yaml
```

### **📋 Sample Manifest (team-setup.yaml)**
```yaml
project: "Web Development Stack"
packages:
  - git
  - nodejs  
  - docker
  - code
```

---

## 🏢 **OMNI ENTERPRISE** — *For Infrastructure Teams*

### **⚡ Install in 60 Seconds**
```bash
curl -sSL https://get-omni.dev/enterprise | sh
```

### **🎯 Enterprise Workflows**
```bash
# Everything Core has, plus:

# Remote server management
omni --ssh prod-servers install security-patch

# Transaction management
omni transaction begin "infrastructure-update"
omni install kubernetes docker-update  
omni transaction commit --verify

# Audit and compliance
omni audit scan --compliance sox
omni audit generate-report --format pdf

# GUI management
omni gui
```

---

## 📦 **Universal Package Management (All Versions)**

### **🔥 The Magic - Same Commands Everywhere**

```bash
# Instead of platform-specific chaos:
sudo apt install firefox      # Linux only
brew install firefox          # macOS only  
winget install firefox        # Windows only

# Use this everywhere:
omni install firefox          # Linux + macOS + Windows ⚡
```

### **🎯 Platform Detection**
```bash
# Omni automatically detects and uses:
• Linux:   apt, dnf, snap, flatpak
• macOS:   brew, mas  
• Windows: winget, chocolatey
```

---

## 🚀 **Real-World Examples**

### **🔥 Daily Developer Workflow**
```bash
# Set up new machine (any platform)
omni install git nodejs docker code
omni install firefox slack zoom

# Done. Universal setup. ⚡
```

### **👥 Team Setup Workflow**
```bash
# Share this with your team
echo "git clone repo && omni manifest install dev-setup.yaml"

# Everyone gets identical environment
# Linux ✓ macOS ✓ Windows ✓
```

### **🏢 Infrastructure Management**
```bash
# Manage multiple servers
omni --ssh web-servers,db-servers install security-update
omni audit generate-compliance-report

# Mission-critical operations ✓
```

---

## 📊 **Performance Comparison**

| **Metric** | **🚀 Lite** | **⚖️ Core** | **🏢 Enterprise** |
|------------|-------------|-------------|-------------------|
| **Build Time** | 18s ⚡ | 45s 📦 | 120s 🏢 |
| **Binary Size** | 865KB | ~10MB | ~50MB |
| **Memory Usage** | <10MB | <30MB | <100MB |
| **Dependencies** | 4 | ~15 | 40+ |
| **Target User** | Daily CLI | Power User | Enterprise |

---

## 🎯 **Next Steps**

### **✅ After Installation**
1. **Verify**: `omni --version`
2. **Test**: `omni search firefox`
3. **Use**: `omni install your-favorite-app`

### **📚 Learn More**
- **Documentation**: [github.com/therealcoolnerd/omni](https://github.com/therealcoolnerd/omni)
- **Examples**: [omni-examples](https://github.com/therealcoolnerd/omni-examples)
- **Support**: [Discord](https://discord.gg/omni) | [GitHub Issues](https://github.com/therealcoolnerd/omni/issues)

### **🚀 Share the Speed**
```bash
# Tell your friends
echo "Universal package management is here: https://get-omni.dev"
```

---

<div align="center">

**🎉 Welcome to the Universal Package Management Revolution! 🎉**

*Built with ❤️ in Rust | Three versions, one vision*

**[⭐ Star on GitHub](https://github.com/therealcoolnerd/omni)** • **[🐛 Report Issues](https://github.com/therealcoolnerd/omni/issues)** • **[💡 Request Features](https://github.com/therealcoolnerd/omni/discussions)**

</div>