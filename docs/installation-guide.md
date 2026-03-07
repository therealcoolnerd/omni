<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Installation Guide**

</div>

---

## Requirements

Omni is built with Rust. Building from source requires:

- **Rust 1.75+** — install from [rustup.rs](https://rustup.rs)
- **Cargo** — included with Rust
- A native package manager already installed on the target platform (apt, winget, brew, etc.)

No other dependencies are required for the default build.

---

## Build from Source

This is the canonical installation method for v1.0.0.

```bash
git clone https://github.com/therealcoolnerd/omni.git
cd omni
cargo build --release
```

The binary is at `target/release/omni` (or `target\release\omni.exe` on Windows).

---

## Install the Binary

### Linux / macOS

```bash
sudo cp target/release/omni /usr/local/bin/omni
```

Or add the build directory to your PATH:

```bash
echo 'export PATH="$PATH:$HOME/omni/target/release"' >> ~/.bashrc
source ~/.bashrc
```

### Windows (PowerShell — elevated)

```powershell
Copy-Item target\release\omni.exe C:\Windows\System32\omni.exe
```

Or add the folder to your user PATH via System Properties → Environment Variables.

---

## Verify

```bash
omni --version
omni --help
```

---

## Optional Feature Flags

### Desktop GUI

Requires platform-native window libraries (automatically provided on most systems).

```bash
cargo build --release --features gui
omni gui
```

### SSH Remote Management

```bash
cargo build --release --features ssh
omni ssh user@host install nginx
```

### Full Build

```bash
cargo build --release --features gui,ssh
```

---

## Platform-Specific Notes

### Linux

Omni detects and uses whichever of the following are present: apt, dnf, pacman, snap, flatpak, zypper, emerge, nix. No manual configuration is needed. Install additional managers through your distribution if needed:

```bash
# Ubuntu/Debian — add Flatpak
sudo apt install flatpak

# Fedora — add Snap
sudo dnf install snapd
```

### macOS

Omni uses `brew` and `mas`. Install Homebrew first if not present:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Windows

Omni uses `winget`, `choco`, and `scoop`. `winget` ships with Windows 10 1809+ and Windows 11. For Chocolatey or Scoop, follow their respective installation guides.

---

## First Run

On first run Omni creates:

- `~/.config/omni/config.yaml` — default configuration
- `~/.local/share/omni/omni.db` — SQLite database (Linux/macOS)
- `%APPDATA%\omni\omni.db` — Windows equivalent

No manual initialization is needed. The first `omni install` will set everything up.

---

## Troubleshooting

### "command not found: omni"

Confirm the binary is in a directory on your `PATH`:

```bash
echo $PATH
which omni        # Linux/macOS
where omni        # Windows
```

### Permission denied on install

`apt`, `dnf`, `pacman`, and `zypper` require elevated privileges. Omni will request them automatically when needed. If you see permission errors, confirm your user can run `sudo` (Linux/macOS) or that you opened an elevated terminal (Windows).

### SSL/TLS errors on Linux

```bash
sudo apt install ca-certificates && sudo update-ca-certificates   # Debian/Ubuntu
sudo dnf install ca-certificates                                    # Fedora
```

### macOS Gatekeeper ("developer cannot be verified")

When building from source and running for the first time on macOS, you may need to approve the binary in System Settings → Privacy & Security.

---

## Updating Omni

Pull the latest source and rebuild:

```bash
cd omni
git pull
cargo build --release
sudo cp target/release/omni /usr/local/bin/omni
```

---

[User Guide](user-guide.md) · [Configuration](../CONFIGURATION.md) · [README](../README.md)
