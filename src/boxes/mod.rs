// src/boxes/mod.rs

/// This module acts as a central point for organizing and exposing all the
/// different "box" modules. Each submodule within `boxes` is responsible for
/// handling a specific type of package manager or installation method (e.g., apt, dnf).
pub mod apt;
pub mod dnf;
pub mod flatpak;
pub mod pacman;
pub mod appimage; // Conceptual support for AppImages.