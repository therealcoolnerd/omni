
use std::process::Command;

pub fn install_with_flatpak(package: &str) {
    println!("🔧 Installing '{}' via flatpak", package);
    let status = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
        .expect("Flatpak failed");
    if status.success() {
        println!("✅ Flatpak installed '{}'", package);
    }
}
