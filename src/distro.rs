use std::fs;

pub fn detect_distro() -> String {
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();
    if contents.contains("Arch") {
        "pacman".to_string()
    } else if contents.contains("Debian") || contents.contains("Ubuntu") {
        "apt".to_string()
    } else if contents.contains("Fedora") {
        "dnf".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} > /dev/null 2>&1", cmd))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
