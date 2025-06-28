use std::process::Command;
use tracing::{info, error};

pub fn install_with_dnf(package: &str) {
    info!("Installing '{}' via dnf", package);
    match Command::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("DNF successfully installed '{}'", package),
        Ok(_) => error!("DNF failed to install '{}'", package),
        Err(e) => error!("Failed to execute dnf: {}", e),
    }
}

pub fn uninstall_with_dnf(package: &str) {
    info!("Removing '{}' via dnf", package);
    match Command::new("sudo")
        .arg("dnf")
        .arg("remove")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => info!("DNF successfully removed '{}'", package),
        Ok(_) => error!("DNF failed to remove '{}'", package),
        Err(e) => error!("Failed to execute dnf: {}", e),
    }
}
