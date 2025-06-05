
mod brain;
mod boxes;
mod manifest;
mod history;
mod distro;

use brain::OmniBrain;
use manifest::OmniManifest;
use std::env;

// Conditionally enable windows_subsystem attribute for release builds on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Example of a Tauri command that could be exposed from Rust to the webview.
// #[tauri::command]
// fn greet(name: &str) -> String { format!("Hello, {}!", name) }

/// Main entry point for the Omni application.
/// This function determines whether to launch the Tauri GUI or operate as a CLI tool
/// based on the command-line arguments provided.
fn main() {
    let args: Vec<String> = env::args().collect();

    // If no arguments are provided (e.g., just `omni`) or if the first argument is "gui",
    // attempt to launch the Tauri GUI application.
    if args.len() == 1 || (args.len() > 1 && args[1] == "gui") {
        // Note: The Tauri setup is currently on hold due to Rustc version compatibility issues.
        // If these issues were resolved, this block would launch the GUI.
        eprintln!("GUI mode initiated, but Tauri setup is currently on hold due to Rustc version issues.");
        eprintln!("To run in CLI mode, provide a command like 'help', 'install <pkg>', etc.");

        // Attempt to run Tauri, which will likely fail if dependencies aren't met,
        // but this structure is kept for future GUI development.
        // tauri::Builder::default()
        //     .invoke_handler(tauri::generate_handler![greet]) // Example for future commands
        //     .run(tauri::generate_context!("./src-tauri/tauri.conf.json"))
        //     .expect("error while running tauri application");
    } else {
        // If arguments are provided and the first isn't "gui", proceed with CLI logic.
        let brain = OmniBrain::new(); // Instantiate the core application logic handler.

        // Define the help message displayed for CLI usage.
        let help_message = r#"
Omni: A universal package manager.

Usage: omni [command] [options]

Available commands:
  install <package_name>         Installs a single package.
  install --from <manifest_path> Installs packages from a manifest file.
  undo                           Undoes the last installation.
  snapshot                       Creates a system snapshot (feature upcoming).
  revert                         Reverts to a previous snapshot (feature upcoming).
  help                           Displays this help message.
"#;

    // Parse the primary command from the arguments.
    match args.get(1).map(|s| s.as_str()) {
        Some("install") => {
            // Handle the "install" command.
            // Check for "--from" flag to install from a manifest file.
            if args.get(2) == Some(&"--from".to_string()) {
                if let Some(path) = args.get(3) {
                    // Load manifest and install.
                    match OmniManifest::from_file(path) {
                        Ok(manifest) => brain.install_from_manifest(manifest),
                        Err(err) => eprintln!("❌ Failed to load manifest: {}", err),
                    }
                } else {
                    eprintln!("⚠️ Please specify the path to a manifest file after --from.");
                    println!("{}", help_message);
                }
            } else if let Some(app) = args.get(2) {
                // Install a single package directly.
                brain.install(app);
            } else {
                // Invalid "install" command usage.
                eprintln!("⚠️ Please specify a package or --from <manifest_path>.");
                println!("{}", help_message);
            }
        }
        Some("undo") => {
            // Handle the "undo" command.
            brain.undo_last();
        }
        Some("snapshot") => {
            // Handle the "snapshot" command (currently a placeholder).
            brain.snapshot();
        }
        Some("revert") => {
            // Handle the "revert" command (currently a placeholder).
            brain.revert();
        }
        Some("help") => {
            // Display the help message.
            println!("{}", help_message);
        }
        None => {
            // No command provided, display the help message.
            println!("{}", help_message);
        }
        _ => {
            // Unknown command provided.
            eprintln!("❌ Unknown command: '{}'", args.get(1).unwrap_or(&String::new()));
            println!("{}", help_message);
            std::process::exit(1); // Exit with an error code for unknown commands.
        }
    }
}
