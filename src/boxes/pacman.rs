use std::process::Command;
use tracing::{info, error};

pub fn install_with_pacman(package: &str) {
    info!("Installing '{}' via pacman", package);
    match Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("Pacman successfully installed '{}'", package),
        Ok(_) => error!("Pacman failed to install '{}'", package),
        Err(e) => error!("Failed to execute pacman: {}", e),
    }
}

pub fn uninstall_with_pacman(package: &str) {
    info!("Removing '{}' via pacman", package);
    match Command::new("sudo")
        .arg("pacman")
        .arg("-R")
        .arg("--noconfirm")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("Pacman successfully removed '{}'", package),
        Ok(_) => error!("Pacman failed to remove '{}'", package),
        Err(e) => error!("Failed to execute pacman: {}", e),
    }
}
