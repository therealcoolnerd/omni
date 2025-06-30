// Linux package managers
pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod zypper;    // openSUSE
pub mod emerge;    // Gentoo
pub mod flatpak;
pub mod snap;
pub mod appimage;

// Cross-platform package managers
pub mod nix;       // NixOS/Nix

// Windows package managers
pub mod winget;
pub mod chocolatey;
pub mod scoop;

// macOS package managers
pub mod brew;
pub mod mas;
