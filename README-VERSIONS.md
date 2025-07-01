# 🎯 Omni Package Manager - Three-Tier Strategy

## Overview

Omni comes in **three distinct versions** to serve different user needs and use cases:

| Version | **Lite** | **Core** | **Enterprise** |
|---------|----------|----------|----------------|
| **Target** | Daily CLI users | Power users | IT teams |
| **Philosophy** | Honda Civic | Toyota Camry | Tesla Model S |
| **Build Time** | < 30s | < 60s | < 120s |
| **Binary Size** | < 1MB | < 15MB | < 50MB |
| **Dependencies** | 4 | < 15 | 40+ |

---

## 🚀 **Omni Lite** - *The Minimalist*

### **Perfect for:** Daily CLI users who want `apt install` to work everywhere

```bash
# The ONLY commands you need:
omni install firefox
omni remove firefox  
omni search browser
omni list
omni update
```

### **Features:**
- ✅ Universal package management (apt, brew, winget)
- ✅ Simple configuration
- ✅ Lightning fast (< 1MB binary)
- ✅ Zero bloat, maximum speed

### **Architecture:**
- 4 dependencies total
- 865KB binary size
- 18-second build time
- < 50ms cold start

### **Use Case:**
```bash
# Developer who just wants package management to work
omni install git nodejs docker
```

---

## ⚖️ **Omni Core** - *The Balanced*

### **Perfect for:** Power users who need reliability with useful extras

```bash
# Everything Lite has, plus:
omni snapshot create backup
omni manifest install dev-setup.yaml
omni update --check-security
omni config set prefer-source brew
```

### **Features:**
- ✅ All Lite features
- ✅ Snapshot management (file-based)
- ✅ Manifest support (YAML setups)
- ✅ Basic security checking
- ✅ Advanced configuration
- ✅ Update management

### **Architecture:**
- < 15 dependencies
- Built from main codebase with `--no-default-features`
- Essential modules only

### **Use Case:**
```bash
# Team lead setting up dev environments
omni manifest install team-setup.yaml
omni snapshot create "pre-deployment"
```

---

## 🏢 **Omni Enterprise** - *The Powerhouse*

### **Perfect for:** Enterprise IT teams and mission-critical environments

```bash
# Everything Core has, plus:
omni gui                           # Desktop interface
omni --ssh user@server install docker
omni transaction begin
omni audit compliance --generate-report
omni resolve dependencies --strategy security
```

### **Features:**
- ✅ All Core features
- ✅ Transaction management with rollback
- ✅ Advanced dependency resolution with conflict detection
- ✅ Audit trails and compliance reporting
- ✅ Remote management via SSH
- ✅ GUI interface for teams
- ✅ Container integration (Docker/Podman)
- ✅ Database-backed state management
- ✅ Enterprise security features

### **Architecture:**
- Current comprehensive codebase
- All advanced modules enabled
- Full security and audit capabilities

### **Use Case:**
```bash
# Enterprise IT managing infrastructure
omni --ssh production-server1,production-server2 install security-update
omni audit generate-compliance-report --format pdf
omni transaction rollback last --confirm
```

---

## 🔧 **Building All Versions**

```bash
# Build all three versions
./build-all.sh

# Or build individually:
cd omni-lite && cargo build --release          # Lite version
cargo build --release --no-default-features    # Core version  
cargo build --release --features full          # Enterprise version
```

---

## 📊 **Performance Comparison**

### **Build Times:**
```
Omni Lite:       18 seconds    ⚡
Omni Core:       45 seconds    📦
Omni Enterprise: 120+ seconds  🏢
```

### **Binary Sizes:**
```
Omni Lite:       865KB         ⚡
Omni Core:       ~10MB         📦
Omni Enterprise: ~50MB         🏢
```

### **Memory Usage:**
```
Omni Lite:       < 10MB        ⚡
Omni Core:       < 30MB        📦
Omni Enterprise: < 100MB       🏢
```

---

## 🎯 **Choosing Your Version**

### **Choose Lite if:**
- You just want `apt install` to work on macOS/Windows
- You value speed and simplicity above all
- You're a developer who wants zero friction
- You don't need advanced features

### **Choose Core if:**
- You want snapshots and manifest support
- You need team coordination features
- You value balance between features and speed
- You want enterprise basics without the bloat

### **Choose Enterprise if:**
- You're managing infrastructure at scale
- You need audit trails and compliance
- You require GUI interfaces for teams
- You need transaction management and rollback
- Security and advanced features are essential

---

## 💡 **Migration Path**

Start with **Lite** → Upgrade to **Core** → Scale to **Enterprise**

All versions share the same:
- Configuration files
- Package manager detection
- Basic command interface
- Cross-platform compatibility

You can seamlessly upgrade between versions as your needs grow.

---

## 🚀 **Quick Start Examples**

### **Lite User:**
```bash
# Daily workflow
omni install firefox git nodejs
omni update
omni list
```

### **Core User:**
```bash
# Team setup workflow
omni snapshot create "clean-slate" 
omni manifest install team-dev-env.yaml
omni update --check-security
omni snapshot create "ready-to-work"
```

### **Enterprise User:**
```bash
# Infrastructure management workflow
omni audit scan --compliance sox
omni transaction begin infrastructure-update
omni --ssh web-servers install security-patches
omni --ssh db-servers install mysql-update
omni transaction commit --verify
omni gui # Open management interface
```

---

**Built with ❤️ in Rust** | Three versions, one vision: Universal package management that scales with your needs.