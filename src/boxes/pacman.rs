use std::process::Command;

pub fn install_with_pacman(package: &str) {
    println!("🔧 Installing '{}' via pacman", package);
    match Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => println!("✅ Pacman installed '{}'", package),
        Ok(_) => eprintln!("❌ Pacman failed to install '{}'", package),
        Err(e) => eprintln!("❌ Failed to execute pacman: {}", e),
    }
}
