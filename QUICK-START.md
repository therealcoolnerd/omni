<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Quick Start**

</div>

---

## 1. Build

```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
```

The binary is at `target/release/omni`.

### Add to PATH

**Linux / macOS:**

```bash
sudo cp target/release/omni /usr/local/bin/
```

**Windows (PowerShell — run as Administrator):**

```powershell
Copy-Item target\release\omni.exe C:\Windows\System32\omni.exe
```

---

## 2. Verify

```bash
omni --version
omni --help
```

---

## 3. Install Your First Package

```bash
omni install git
```

Omni detects the current platform, selects the appropriate package manager, and runs the install. No flags, no OS branches.

---

## 4. Core Commands

```bash
omni install <package>        # Install
omni remove  <package>        # Remove
omni search  <query>          # Search all available managers
omni info    <package>        # Package details
omni list                     # Installed packages
omni update --all             # Update everything
```

---

## 5. History and Safety

Every operation is logged. You can review and reverse it:

```bash
omni history show             # See what has been installed/removed
omni history undo             # Reverse the last operation

omni snapshot create "clean"  # Save current state
omni snapshot revert "clean"  # Restore to saved state
```

---

## 6. Optional Features

### Desktop GUI

```bash
cargo build --release --features gui
omni gui
```

### SSH Remote Management

```bash
cargo build --release --features ssh
omni ssh user@server install nginx
```

### Web Dashboard

```bash
omni web --port 3000
# Open http://127.0.0.1:3000
```

---

## 7. Mock Mode (Safe Testing)

Test any command without touching the system:

```bash
omni --mock install firefox
omni --mock update --all
```

---

## What's Next

- Full command reference: [docs/user-guide.md](docs/user-guide.md)
- Build options and platforms: [docs/installation-guide.md](docs/installation-guide.md)
- Configuration options: [CONFIGURATION.md](CONFIGURATION.md)
- Full feature overview: [README.md](README.md)
