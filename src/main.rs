mod boxes;
mod brain;
mod distro;
mod gui;
mod history;
mod manifest;

use brain::OmniBrain;
use manifest::OmniManifest;
use std::env;

fn print_help() {
    println!("Omni - Universal Linux Installer");
    println!("Usage:");
    println!("  omni install <package>");
    println!("  omni install --from <manifest>");
    println!("  omni undo");
    println!("  omni snapshot");
    println!("  omni revert");
    println!("  omni gui");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let brain = OmniBrain::new();

    match args.get(1).map(|s| s.as_str()) {
        Some("install") => {
            if args.get(2) == Some(&"--from".to_string()) {
                if let Some(path) = args.get(3) {
                    match OmniManifest::from_file(path) {
                        Ok(manifest) => brain.install_from_manifest(manifest),
                        Err(err) => eprintln!("❌ Failed to load manifest: {}", err),
                    }
                }
            } else if let Some(app) = args.get(2) {
                brain.install(app);
            } else {
                eprintln!("⚠️ Please specify a package or manifest.");
            }
        }
        Some("undo") => brain.undo_last(),
        Some("snapshot") => brain.snapshot(),
        Some("revert") => brain.revert(),
        Some("gui") => gui::launch_gui(),
        Some("help") | Some("--help") | Some("-h") | None => print_help(),
        _ => {
            eprintln!("❌ Unknown command.");
            print_help();
        }
    }
}
