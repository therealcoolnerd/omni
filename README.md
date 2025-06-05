# 🌌 omni — Universal Linux Installer Engine

**One CLI. One System. One Omni.**

Omni is the future of Linux package installation — built to unify `.deb`, `.rpm`, Flatpak, AppImage, and beyond — under a single command-line interface that feels like magic.

With modular **Omni Boxes**, a secure **Omni Brain**, and the high-speed **Omni Engine**, it’s designed to be your *forever installer* — whether you're on Ubuntu, Arch, Fedora, or something off the grid.

## ⚡️ Features
- 🔀 **Universal Installer Engine:** Core logic for installing packages from various sources. Currently supports direct installs via system package managers like `apt`, `pacman`, `dnf`, and Flatpak through manifest files.
- 🔁 **Rollback Installs:** Undo previous installations via `omni undo`.
- 📜 **Comprehensive Help Command:** Detailed usage information available via `omni help`.
- 🛠️ **Improved Error Handling:** More robust error handling and clearer user feedback during operations.
- 💾 **Standardized History:** Installation history is now stored in a user-specific configuration directory (`~/.config/omni/history.json`).
- 🖼️ **AppImage Support (Conceptual):** Basic framework for AppImage support via manifests is in place (currently in a conceptual, non-functional stage).
- 🧠 **Omni Brain:** Smart logic for resolving dependencies, packages, and rollbacks (partially implemented).
- 📦 **Omni Boxes:** Pluggable backend modules for each package format or distribution.
- 🧾 **Omni Manifest:** Define installations in YAML, JSON, or TOML for projects (`omni install --from <manifest_path>`).
- 📸 **Snapshot Feature (Planned):** The `omni snapshot` command is a placeholder for a planned feature to save the current state of your installed packages.
- ⏪ **Revert Feature (Planned):** The `omni revert` command is a placeholder for a planned feature to restore package states from a previously created snapshot.
- 🔒 **Security (Planned):** Future plans include GPG/PGP signature checks and hash verification.


## 🚀 Getting Started

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/therealcoolnerd/omni.git
    cd omni
    ```
2.  **Build the project:**
    ```bash
    cargo build
    ```
3.  **Run Omni:**
    *   To see all available commands:
        ```bash
        ./target/debug/omni help
        ```
    *   Example: Install packages from a manifest file:
        ```bash
        ./target/debug/omni install --from omni.manifest.yaml
        ```
    *   **Note on Privileges:** Most installation and uninstallation operations performed by Omni require `sudo` privileges, as they interact with system-level package managers. Omni will typically invoke `sudo` internally for these commands.

## 🔥 Optional GUI (Omni Flame)

Omni Flame is envisioned as a sleek, cross-platform graphical frontend for Omni, providing an intuitive way to manage packages, view history, and interact with all of Omni's features.

**Current Status:** Development of Omni Flame (using Tauri) is currently **on hold**. This is due to encountering Rust compiler version compatibility issues within the development/testing environment that prevent the successful compilation of Tauri and its dependencies. We plan to resume GUI development when these toolchain issues are resolved or a more compatible environment is available.

## 📜 License
GNU AGPLv3
