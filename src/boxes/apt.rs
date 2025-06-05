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

pub fn uninstall_with_apt(package: &str) {
    println!("🗑 Removing '{}' via apt", package);
    match Command::new("sudo")
        .arg("apt")
        .arg("remove")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => println!("✅ APT removed '{}'", package),
        Ok(_) => eprintln!("❌ APT failed to remove '{}'", package),
        Err(e) => eprintln!("❌ Failed to execute apt: {}", e),
    }
}
