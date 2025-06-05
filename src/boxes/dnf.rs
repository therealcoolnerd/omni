use std::process::Command;

pub fn install_with_dnf(package: &str) {
    println!("🔧 Installing '{}' via dnf", package);
    match Command::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => println!("✅ DNF installed '{}'", package),
        Ok(_) => eprintln!("❌ DNF failed to install '{}'", package),
        Err(e) => eprintln!("❌ Failed to execute dnf: {}", e),
    }
}
