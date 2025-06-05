
use std::process::Command;

pub fn install_with_pacman(package: &str) {
    println!("🔧 Installing '{}' via pacman", package);
    let status = Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(package)
        .status()
        .expect("Pacman failed");
    if status.success() {
        println!("✅ Pacman installed '{}'", package);
    }
}
