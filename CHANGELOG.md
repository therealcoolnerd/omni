# 📋 Omni Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2025-12-23

### ✨ **New Features**
- **Premium Web Dashboard**: A completely new React-based web interface for managing your system.
  - Interactive dashboard with system stats and usage graphs.
  - Visual package management (search, install, remove).
  - Modern "Deep Space" theme with glassmorphism effects.
- **REST API Server**: New `omni web` command launches a local Axum server.
  - Endpoints for system info, package management, and search.
  - Cors protection and secure backend integration.
- **Hardware Integration**: Improved detection for server-grade hardware (Dell, HP, Supermicro).

### 🛠️ **Improvements**
- **Documentation**: Overhauled README with better SEO and clear "What Works" tables.
- **Performance**: Optimized package detection logic for faster startup.
- **Security**: Hardened dependency chain and pinned critical versions.
- **Code Quality**: Major cleanup of `main.rs` and modularization of components.

### 🐛 **Bug Fixes**
- Fixed build issues related to missing module declarations in `src/main.rs`.
- Resolved `node-forge` security vulnerability in web app dependencies.
- Corrected version inconsistencies across crates.

---

## [0.2.0] - 2025-08-15
### Added
- Initial support for Windows `winget`.
- Basic SSH remote management implementation.
- SQLite database integration for history tracking.

## [0.1.0] - 2025-01-10
### Added
- Initial release.
- Support for `apt`, `brew`, and `pacman`.
- Basic CLI structure using `clap`.
