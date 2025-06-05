use crate::boxes::{apt, dnf, flatpak, pacman, appimage};
use crate::distro;
use crate::history;
use crate::manifest::OmniManifest;

/// `OmniBrain` is the central struct orchestrating the application's logic.
/// It handles package installations, uninstallation, history management,
/// and interaction with different package manager "boxes".
pub struct OmniBrain;

impl OmniBrain {
    /// Creates a new instance of `OmniBrain`.
    pub fn new() -> Self {
        OmniBrain
    }

    /// Installs a single application using the detected native package manager.
    ///
    /// This method determines the Linux distribution and uses the corresponding
    /// box (e.g., apt, pacman, dnf) to install the application. If the installation
    /// is successful, it records the operation in the history.
    ///
    /// # Arguments
    /// * `app` - The name of the application to install.
    pub fn install(&self, app: &str) {
        println!("🔥 Installing '{}'", app);
        let distro_type = distro::detect_distro(); // Detect the host distribution type.
        let install_result = match distro_type.as_str() {
            "apt" => apt::install_with_apt(app),
            "pacman" => pacman::install_with_pacman(app), // Attempt Pacman install.
            "dnf" => dnf::install_with_dnf(app),       // Attempt DNF install.
            other => {
                // If the detected distribution is not supported, print an error and return.
                eprintln!("❌ Unsupported distro: {}", other);
                return;
            }
        };

        // Process the result of the installation attempt.
        match install_result {
            Ok(_) => {
                // If installation was successful, try to save it to history.
                if let Err(e) = history::save_install(app, &distro_type) {
                    eprintln!("Error saving history: {}", e); // Report history save errors.
                }
            }
            Err(e) => {
                // If installation failed, print the error message from the box.
                eprintln!("{}", e);
            }
        }
    }

    /// Installs applications from a given `OmniManifest`.
    ///
    /// This method iterates through the apps defined in the manifest,
    /// attempts to install them using their specified box type, and uses a
    /// fallback mechanism to the native package manager if enabled and the
    /// primary box type fails or is unavailable.
    ///
    /// # Arguments
    /// * `manifest` - The `OmniManifest` containing apps to install.
    pub fn install_from_manifest(&self, manifest: OmniManifest) {
        // Determine if fallback to native package manager is enabled in the manifest metadata.
        let fallback = manifest
            .meta
            .as_ref()
            .and_then(|m| m.distro_fallback)
            .unwrap_or(false);

        for app in manifest.apps {
            let mut handled = false; // Flag to track if an app's installation was successful.

            // Closure to encapsulate the logic of attempting an install and recording it.
            // Takes an installation function, package name, and box type.
            // Returns Ok(true) if install succeeds, Err(message) if it fails.
            let install_and_record = |install_fn: fn(&str) -> Result<(), String>,
                                      pkg_name: &str,
                                      box_type: &str|
             -> Result<bool, String> {
                match install_fn(pkg_name) {
                    Ok(_) => {
                        // Installation successful, now try to save to history.
                        if let Err(e) = history::save_install(pkg_name, box_type) {
                            eprintln!("Error saving history for {}: {}", pkg_name, e);
                            // TODO: Decide if a history save error should mark `handled` as false.
                            // Current behavior: Installation is considered successful even if history save fails.
                            // The error is reported, but the overall operation for this app might still be `Ok(true)`.
                        }
                        Ok(true) // Installation (not necessarily history save) was successful.
                    }
                    Err(e) => {
                        // Installation failed.
                        eprintln!("{}", e); // Print the error from the installation function.
                        Err(e) // Propagate the installation error.
                    }
                }
            };

            // Attempt to install the app using its specified box type.
            match app.box_type.as_str() {
                "apt" if distro::command_exists("apt") => {
                    if install_and_record(apt::install_with_apt, &app.name, "apt").is_ok() {
                        handled = true;
                    }
                }
                "pacman" if distro::command_exists("pacman") => {
                    if install_and_record(pacman::install_with_pacman, &app.name, "pacman").is_ok() {
                        handled = true;
                    }
                }
                "dnf" if distro::command_exists("dnf") => {
                    if install_and_record(dnf::install_with_dnf, &app.name, "dnf").is_ok() {
                        handled = true;
                    }
                }
                "flatpak" if distro::command_exists("flatpak") => {
                    let name_to_install = app.source.as_deref().unwrap_or(&app.name);
                    if install_and_record(flatpak::install_with_flatpak, name_to_install, "flatpak").is_ok() {
                        handled = true;
                    }
                }
                "appimage" => { // Assuming AppImage doesn't need a specific command_exists check for conceptual phase
                    let source_for_appimage = app.source.as_deref().unwrap_or(&app.name);
                    if install_and_record(appimage::install_with_appimage, source_for_appimage, "appimage").is_ok() {
                        handled = true;
                    }
                }
                _ => { // Box type not available or not recognized for direct install
                    // handled remains false; will attempt fallback if enabled.
                }
            }

            // If the app wasn't handled by its specified box type, attempt fallback if enabled.
            if !handled {
                if fallback {
                    println!("↪️ Fallback for {}: trying native package manager...", app.name);
                    let distro_type = distro::detect_distro(); // Detect native distro for fallback.
                    let fallback_install_result = match distro_type.as_str() {
                        "apt" if distro::command_exists("apt") => {
                            install_and_record(apt::install_with_apt, &app.name, &distro_type)
                        }
                        "pacman" if distro::command_exists("pacman") => {
                            install_and_record(pacman::install_with_pacman, &app.name, &distro_type)
                        }
                        "dnf" if distro::command_exists("dnf") => {
                            install_and_record(dnf::install_with_dnf, &app.name, &distro_type)
                        }
                        // Note: No AppImage fallback here as it's not a "native" system package manager.
                        other => {
                            eprintln!("❌ Native fallback failed: Unsupported distro '{}'", other);
                            Err(format!("Unsupported distro for fallback: {}", other))
                        }
                    };
                    // If fallback installation was successful, mark as handled.
                    if fallback_install_result.is_ok() {
                        handled = true;
                    } else {
                        eprintln!(
                            "❌ Fallback installation failed for {}. Specified box: '{}'.",
                            app.name, app.box_type
                        );
                    }
                }
            }

            // If the app is still not handled after primary attempt and potential fallback,
            // print a final failure message for this app.
            if !handled {
                eprintln!(
                    "❌ Failed to install '{}' using box '{}' and fallback (if enabled).",
                    app.name, app.box_type
                );
            }
        }
    }

    /// Undoes the last recorded installation.
    ///
    /// This method relies on `history::undo_last_install` to perform the
    /// necessary uninstall operations and update the history file.
    pub fn undo_last(&self) {
        if let Err(e) = history::undo_last_install() {
            eprintln!("Error during undo operation: {}", e);
        }
    }

    /// Placeholder for the snapshot feature.
    /// Prints an informative message about the feature being planned.
    pub fn snapshot(&self) {
        println!("📸 The 'snapshot' feature is planned for a future release. It will allow you to save the current state of your installed packages. Stay tuned!");
    }

    /// Placeholder for the revert feature.
    /// Prints an informative message about the feature being planned.
    pub fn revert(&self) {
        println!("⏪ The 'revert' feature is planned for a future release. It will allow you to restore package states from a previously created snapshot. Stay tuned!");
    }
}
