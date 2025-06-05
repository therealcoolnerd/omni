use std::process::Command;

/// Installs a package using Flatpak.
///
/// This function attempts to install the specified Flatpak application using
/// `flatpak install --noninteractive <package>`. Note that Flatpak installations
/// are typically user-specific unless `sudo` is used or system-wide remotes are configured.
/// This function, as written, does not use `sudo`.
///
/// # Arguments
/// * `package` - The name or ID of the Flatpak application to install (e.g., `org.gimp.GIMP`).
///
/// # Returns
/// * `Ok(())` if the application was installed successfully.
/// * `Err(String)` with an error message if the installation failed or the command couldn't be executed.
pub fn install_with_flatpak(package: &str) -> Result<(), String> {
    println!("🔧 Installing '{}' via flatpak", package);
    // Construct and execute the `flatpak install --noninteractive` command.
    match Command::new("flatpak") // Flatpak commands usually don't require sudo for user installs.
        .arg("install")
        .arg("--noninteractive") // Ensures no interactive prompts.
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully.
            println!("✅ Flatpak installed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "Flatpak failed to install '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute flatpak: {}", e))
        }
    }
}

/// Uninstalls a package using Flatpak.
///
/// This function attempts to remove the specified Flatpak application using
/// `flatpak uninstall --noninteractive <package>`.
///
/// # Arguments
/// * `package` - The name or ID of the Flatpak application to uninstall.
///
/// # Returns
/// * `Ok(())` if the application was uninstalled successfully.
/// * `Err(String)` with an error message if the uninstallation failed or the command couldn't be executed.
pub fn uninstall_with_flatpak(package: &str) -> Result<(), String> {
    println!("🗑 Removing '{}' via flatpak", package);
    // Construct and execute the `flatpak uninstall --noninteractive` command.
    match Command::new("flatpak")
        .arg("uninstall")
        .arg("--noninteractive") // Ensures no interactive prompts.
        .arg(package)
        .status()
    {
        Ok(status) if status.success() => {
            // Command executed successfully.
            println!("✅ Flatpak removed '{}'", package);
            Ok(())
        }
        Ok(status) => {
            // Command executed but reported a failure.
            Err(format!(
                "Flatpak failed to remove '{}': status code {:?}",
                package,
                status.code()
            ))
        }
        Err(e) => {
            // Failed to execute the command.
            Err(format!("Failed to execute flatpak: {}", e))
        }
    }
}
