use std::process::Command;

/// Installs a package using the DNF package manager.
///
/// This function attempts to install the specified package using `sudo dnf install -y <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to install.
///
/// # Returns
/// * `Ok(())` if the package was installed successfully.
/// * `Err(String)` with an error message if the installation failed or the command couldn't be executed.
pub fn install_with_dnf(package: &str) -> Result<(), String> {
    println!("🔧 Installing '{}' via dnf", package);
    // Construct and execute the `sudo dnf install -y` command.
    match Command::new("sudo")
        .arg("dnf")
        .arg("install") // DNF command for installing packages.
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success.
            println!("✅ DNF installed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "DNF failed to install '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute dnf: {}", e))
        }
    }
}

/// Uninstalls a package using the DNF package manager.
///
/// This function attempts to remove the specified package using `sudo dnf remove -y <package>`.
/// It prints status messages to the console and returns a Result indicating success or failure.
///
/// # Arguments
/// * `package` - The name of the package to uninstall.
///
/// # Returns
/// * `Ok(())` if the package was uninstalled successfully.
/// * `Err(String)` with an error message if the uninstallation failed or the command couldn't be executed.
pub fn uninstall_with_dnf(package: &str) -> Result<(), String> {
    println!("🗑 Removing '{}' via dnf", package);
    // Construct and execute the `sudo dnf remove -y` command.
    match Command::new("sudo")
        .arg("dnf")
        .arg("remove") // DNF command for removing packages.
        .arg("-y")
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully and reported success.
            println!("✅ DNF removed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "DNF failed to remove '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute dnf: {}", e))
        }
    }
}
