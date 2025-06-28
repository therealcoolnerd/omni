use std::process::Command;
use tracing::{info, error};

pub fn install_with_flatpak(package: &str) {
    info!("Installing '{}' via flatpak", package);
    match Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("Flatpak successfully installed '{}'", package),
        Ok(_) => error!("Flatpak failed to install '{}'", package),
        Err(e) => error!("Failed to execute flatpak: {}", e),
    }
}

pub fn uninstall_with_flatpak(package: &str) {
    info!("Removing '{}' via flatpak", package);
    match Command::new("flatpak")
        .arg("uninstall")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("Flatpak successfully removed '{}'", package),
        Ok(_) => error!("Flatpak failed to remove '{}'", package),
        Err(e) => error!("Failed to execute flatpak: {}", e),
    }
}
