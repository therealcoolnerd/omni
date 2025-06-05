use std::process::Command;

pub fn install_with_flatpak(package: &str) {
    println!("🔧 Installing '{}' via flatpak", package);
    match Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => println!("✅ Flatpak installed '{}'", package),
        Ok(_) => eprintln!("❌ Flatpak failed to install '{}'", package),
        Err(e) => eprintln!("❌ Failed to execute flatpak: {}", e),
    }
}
