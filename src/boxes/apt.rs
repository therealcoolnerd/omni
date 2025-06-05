
use std::process::Command;

pub fn install_with_apt(package: &str) {
    println!("🔧 Installing '{}' via apt", package);
    let status = Command::new("sudo")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
        .expect("APT failed");
    if status.success() {
        println!("✅ APT installed '{}'", package);
    }
}
