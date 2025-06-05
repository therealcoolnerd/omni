use std::process::Command;

/// Installs a package using the Pacman package manager.
///
/// This function attempts to install the specified package using `sudo pacman -S --noconfirm <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to install.
///
/// # Returns
/// * `Ok(())` if the package was installed successfully.
/// * `Err(String)` with an error message if the installation failed or the command couldn't be executed.
pub fn install_with_pacman(package: &str) -> Result<(), String> {
    println!("🔧 Installing '{}' via pacman", package);
    // Construct and execute the `sudo pacman -S --noconfirm` command.
    match Command::new("sudo")
        .arg("pacman")
        .arg("-S") // Synchronize packages (install).
        .arg("--noconfirm")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success.
            println!("✅ Pacman installed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "Pacman failed to install '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute pacman: {}", e))
        }
    }
}

/// Uninstalls a package using the Pacman package manager.
///
/// This function attempts to remove the specified package using `sudo pacman -R --noconfirm <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to uninstall.
///
/// # Returns
/// * `Ok(())` if the package was uninstalled successfully.
/// * `Err(String)` with an error message if the uninstallation failed or the command couldn't be executed.
pub fn uninstall_with_pacman(package: &str) -> Result<(), String> {
    println!("🗑 Removing '{}' via pacman", package);
    // Construct and execute the `sudo pacman -R --noconfirm` command.
    match Command::new("sudo")
        .arg("pacman")
        .arg("-R") // Remove packages.
        .arg("--noconfirm")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success.
            println!("✅ Pacman removed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "Pacman failed to remove '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute pacman: {}", e))
        }
    }
}
