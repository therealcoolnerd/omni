// Linux package managers
pub mod appimage;
pub mod apt;
pub mod dnf;
pub mod emerge; // Gentoo
pub mod flatpak;
pub mod pacman;
pub mod snap;
pub mod zypper; // openSUSE

// Cross-platform package managers
pub mod nix; // NixOS/Nix

// Windows package managers
pub mod chocolatey;
pub mod scoop;
pub mod winget;

// macOS package managers
pub mod brew;
pub mod mas;
