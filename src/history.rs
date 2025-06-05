
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;
use crate::boxes::{apt, dnf, flatpak, pacman, appimage};

/// Retrieves the path to the history file (`history.json`).
/// This path is typically `~/.config/omni/history.json`.
///
/// It ensures that the `omni` configuration subdirectory exists.
///
/// # Returns
/// * `Ok(PathBuf)` with the full path to the history file.
/// * `Err(std::io::Error)` if the config directory cannot be found or created.
fn get_history_file_path() -> Result<PathBuf, std::io::Error> {
    match dirs::config_dir() { // Uses the `dirs` crate to find the user's config directory.
        Some(mut path) => {
            path.push("omni"); // Append our application's specific config directory name.
            create_dir_all(&path)?; // Ensure this directory exists, creating it if necessary.
            path.push("history.json"); // Append the history filename.
            Ok(path)
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "User config directory not found", // Error if the base config directory isn't available.
        )),
    }
}

/// Represents a single installation record in the history.
#[derive(Serialize, Deserialize, Debug)]
pub struct InstallRecord {
    /// The name of the package that was installed.
    /// For AppImages, this might be the source URL or a user-defined name.
    pub package: String,
    /// The type of "box" (package manager or method) used for installation (e.g., "apt", "appimage").
    pub box_type: String,
    /// Timestamp of when the installation occurred, in RFC3339 format.
    pub timestamp: String,
}

/// Saves a record of an installation to the history file.
///
/// Appends the new record to the existing history (or creates a new history file).
///
/// # Arguments
/// * `package` - Name of the package installed.
/// * `box_type` - The type of box used (e.g., "apt").
///
/// # Returns
/// * `Ok(())` if saving was successful.
/// * `Err(std::io::Error)` if there was an I/O error during file operations or path resolution.
pub fn save_install(package: &str, box_type: &str) -> Result<(), std::io::Error> {
    let timestamp = chrono::Utc::now().to_rfc3339(); // Get current UTC time for the record.
    let record = InstallRecord {
        package: package.to_string(), // Convert &str to String for ownership.
        box_type: box_type.to_string(), // Convert &str to String.
        timestamp,                      // Timestamp from chrono.
    };

    let history_file_path = get_history_file_path()?; // Get the dynamic path to the history file.
    let mut history = load_history()?; // Load existing history.
    history.push(record); // Add the new record.

    // Open the history file for writing (create if doesn't exist, truncate to overwrite).
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(history_file_path)?;

    // Serialize the updated history vector to the file in a pretty JSON format.
    serde_json::to_writer_pretty(file, &history)?;
    Ok(())
}

/// Loads the installation history from the history file.
///
/// # Returns
/// * `Ok(Vec<InstallRecord>)` containing all history records. Returns an empty vector
///   if the history file doesn't exist or if there's a deserialization error (e.g., empty/corrupt file).
/// * `Err(std::io::Error)` if there's an I/O error opening or reading the file,
///   other than `NotFound`.
pub fn load_history() -> Result<Vec<InstallRecord>, std::io::Error> {
    let history_file_path = get_history_file_path()?;
    let file_result = OpenOptions::new().read(true).open(history_file_path);

    match file_result {
        Ok(f) => {
            let reader = BufReader::new(f); // Use a buffered reader for efficiency.
            // Attempt to deserialize JSON from the reader.
            // `unwrap_or_default()` returns an empty Vec if deserialization fails (e.g., empty file, corrupt JSON),
            // which is acceptable behavior for a missing or invalid history.
            Ok(serde_json::from_reader(reader).unwrap_or_default())
        }
        // If the history file is not found, it's not an error; it just means no history exists yet. Return empty vec.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        // For any other I/O error, propagate it.
        Err(e) => Err(e),
    }
}

/// Undoes the last installation recorded in the history.
///
/// This function:
/// 1. Loads the history.
/// 2. Removes the most recent installation record.
/// 3. Attempts to call the appropriate uninstallation function based on the record's box type.
/// 4. Saves the modified history (even if the uninstallation command fails, to reflect the attempt).
///
/// # Returns
/// * `Ok(())` if the operation completed (history was loaded and saved, uninstall attempted).
/// * `Err(std::io::Error)` if there was an I/O error loading or saving the history file.
pub fn undo_last_install() -> Result<(), std::io::Error> {
    let history_file_path = get_history_file_path()?;
    let mut history = load_history()?; // Load current installation history.

    if let Some(last) = history.pop() { // Remove the last installation record.
        println!("🧹 Undoing '{}' via '{}'", last.package, last.box_type);

        // Attempt to uninstall the package using the corresponding box module.
        let uninstall_result = match last.box_type.as_str() {
            "apt" => apt::uninstall_with_apt(&last.package),
            "pacman" => pacman::uninstall_with_pacman(&last.package),
            "dnf" => dnf::uninstall_with_dnf(&last.package),
            "flatpak" => flatpak::uninstall_with_flatpak(&last.package),
            "appimage" => appimage::uninstall_with_appimage(&last.package), // Conceptual AppImage uninstall.
            other => {
                eprintln!("❌ Unknown box type for undo: '{}'", other);
                // Treat as an error for consistent handling, though the primary effect is just a message.
                // The history will still be saved with this item removed.
                Err(format!("Unknown box type: {}", other))
            }
        };

        // If the uninstallation command itself failed (e.g., package not found by apt),
        // print an error message. The history record is already popped and will be saved as such.
        if let Err(e) = uninstall_result {
            eprintln!(
                "⚠️ Error during uninstall of '{}' with box type '{}': {}. History record is removed regardless.",
                last.package, last.box_type, e
            );
        }

        // Save the modified history (with the last item removed).
        let file = OpenOptions::new()
            .write(true)
            .truncate(true) // Overwrite the file with the new, shorter history.
            .open(history_file_path)?;
        serde_json::to_writer_pretty(file, &history)?;
    } else {
        // If there was no history to begin with.
        println!("📭 No install history found.");
    }
    Ok(())
}
