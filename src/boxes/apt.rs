use std::process::Command;

pub fn install_with_apt(package: &str) {
    println!("🔧 Installing '{}' via apt", package);
    match Command::new("sudo")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => println!("✅ APT installed '{}'", package),
        Ok(_) => eprintln!("❌ APT failed to install '{}'", package),
        Err(e) => eprintln!("❌ Failed to execute apt: {}", e),
    }
}
