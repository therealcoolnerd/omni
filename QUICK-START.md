# ⚡ **Omni Quick Start Guide**

Get Omni running in under 5 minutes.

## 🛠️ **Installation**

### **Build from Source**
```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
```

### **Copy to PATH** 
```bash
# Copy the binary to your PATH
sudo cp target/release/omni /usr/local/bin/

# Or add to your shell profile
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.bashrc
```

## 🚀 **Basic Usage**

### **Install Packages**
```bash
omni install firefox        # Install Firefox
omni install git nodejs     # Install multiple packages
```

### **Search Packages**
```bash
omni search browser         # Find browser packages
omni search "video editor"  # Search with spaces
```

### **Manage Packages**
```bash
omni list                   # List installed packages
omni remove firefox         # Remove a package
omni update                 # Update all packages
omni info docker           # Get package information
```

## 🎯 **Optional Features**

### **GUI Interface**
```bash
# Build with GUI support
cargo build --release --features gui

# Launch GUI
omni gui
```

### **SSH Remote Management**
```bash
# Build with SSH support
cargo build --release --features ssh

# Install on remote server
omni ssh user@server install nginx
```

## ✅ **Verify Installation**

```bash
omni --version              # Should show version
omni list                   # Should show installed packages
```

That's it! You now have universal package management across platforms.

## 📚 **Next Steps**

- Read the [main README](README.md) for full documentation
- Report issues on [GitHub](https://github.com/therealcoolnerd/omni/issues)
- Contribute to the project