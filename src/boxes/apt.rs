use std::process::Command;

/// Installs a package using the APT package manager.
///
/// This function attempts to install the specified package using `sudo apt install -y <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to install.
///
/// # Returns
/// * `Ok(())` if the package was installed successfully.
/// * `Err(String)` with an error message if the installation failed or the command couldn't be executed.
pub fn install_with_apt(package: &str) -> Result<(), String> {
    println!("🔧 Installing '{}' via apt", package);
    // Construct and execute the `sudo apt install -y` command.
    match Command::new("sudo")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success (exit code 0).
            println!("✅ APT installed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure (non-zero exit code).
            Err(format!(
                "APT failed to install '{}': status code {:?}",
                package,
                status.code() // Include the exit code in the error.
            ))
        }
        Err(e) => {
            // Failed to even execute the command (e.g., `sudo` or `apt` not found).
            Err(format!("Failed to execute apt: {}", e))
        }
    }
}

/// Uninstalls a package using the APT package manager.
///
/// This function attempts to remove the specified package using `sudo apt remove -y <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to uninstall.
///
/// # Returns
/// * `Ok(())` if the package was uninstalled successfully.
/// * `Err(String)` with an error message if the uninstallation failed or the command couldn't be executed.
pub fn uninstall_with_apt(package: &str) -> Result<(), String> {
    println!("🗑 Removing '{}' via apt", package);
    // Construct and execute the `sudo apt remove -y` command.
    match Command::new("sudo")
        .arg("apt")
        .arg("remove")
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success.
            println!("✅ APT removed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "APT failed to remove '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute apt: {}", e))
        }
    }
}
