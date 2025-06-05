# 🌌 omni — Universal Linux Installer Engine

**One CLI. One System. One Omni.**

Omni is the future of Linux package installation — built to unify `.deb`, `.rpm`, Flatpak, AppImage, and beyond — under a single command-line interface that feels like magic.

With modular **Omni Boxes**, a secure **Omni Brain**, and the high-speed **Omni Engine**, it’s designed to be your *forever installer* — whether you're on Ubuntu, Arch, Fedora, or something off the grid.

## ⚡️ Features
- 🔀 Universal Installer — install `.deb`, `.rpm`, `.pkg.tar.zst`, AppImage, Flatpak, etc.
- 🔁 Rollback Installs — undo installs with snapshots via `omni undo`
- 🔒 Security First — GPG/PGP sig checks, hash verification
- 🧠 Omni Brain — smart logic for resolving dependencies, packages, and rollbacks
- 📦 Omni Boxes — plug-and-play backend modules for each format or distro
- 🧾 Omni Manifest — define installs in YAML, JSON, or TOML for projects
- 🖥️ Optional GUI — Omni Flame, a sleek cross-platform frontend (coming soon)

## 🚀 Getting Started
```bash
git clone https://github.com/yourname/omni.git
cd omni
cargo build
./target/debug/omni install --from omni.manifest.yaml
```

## 📜 License
GNU AGPLv3
