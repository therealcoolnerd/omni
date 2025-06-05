use std::fs;

/// Detects the Linux distribution based on the content of `/etc/os-release`.
///
/// This function reads `/etc/os-release` and checks for specific keywords to
/// identify common distributions and their associated package managers.
///
/// # Returns
/// A string representing the primary package manager or a distribution group:
/// - `"pacman"` for Arch Linux.
/// - `"apt"` for Debian, Ubuntu, and derivatives.
/// - `"dnf"` for Fedora.
/// - `"unknown"` if the distribution cannot be determined from the file.
pub fn detect_distro() -> String {
    // Attempt to read the /etc/os-release file. If reading fails, default to an empty string.
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();

    // Check for keywords associated with different distributions.
    if contents.contains("Arch") {
        "pacman".to_string()
    } else if contents.contains("Debian") || contents.contains("Ubuntu") {
        // Covers Debian, Ubuntu, and many derivatives like Mint, Pop!_OS.
        "apt".to_string()
    } else if contents.contains("Fedora") {
        "dnf".to_string()
    } else {
        // If no specific distribution is identified.
        "unknown".to_string()
    }
}

/// Checks if a given command exists in the system's PATH.
///
/// It uses the `command -v` shell command, which is a POSIX standard way
/// to check for command existence without executing it.
///
/// # Arguments
/// * `cmd` - The name of the command to check (e.g., "apt", "flatpak").
///
/// # Returns
/// `true` if the command exists and is executable, `false` otherwise.
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("sh") // Use the system shell.
        .arg("-c") // Execute the following string as a command.
        // `command -v cmd` checks if `cmd` is a known command.
        // Output is redirected to /dev/null to suppress it.
        .arg(format!("command -v {} > /dev/null 2>&1", cmd))
        .status() // Execute the command and get its exit status.
        .map(|s| s.success()) // Map the Result<ExitStatus, Error> to Result<bool, Error>. True if exit code was 0.
        .unwrap_or(false) // If Command::new or status() failed (e.g., `sh` not found), default to false.
}
