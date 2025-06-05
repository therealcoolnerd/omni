
mod brain;
mod boxes;
mod manifest;
mod history;
mod distro;

use brain::OmniBrain;
use manifest::OmniManifest;
use std::env;

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
        _ => eprintln!("❌ Unknown or missing command."),
    }
}
