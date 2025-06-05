
use crate::boxes::{apt, flatpak, pacman};
use crate::manifest::OmniManifest;
use crate::history;
use crate::distro;

pub struct OmniBrain;

impl OmniBrain {
    pub fn new() -> Self { OmniBrain }

    pub fn install(&self, app: &str) {
        println!("🔥 Installing '{}'", app);
        match distro::detect_distro().as_str() {
            "apt" => {
                apt::install_with_apt(app);
                history::save_install(app, "apt");
            }
            "pacman" => {
                pacman::install_with_pacman(app);
                history::save_install(app, "pacman");
            }
            other => {
                eprintln!("❌ Unsupported distro: {}", other);
            }
        }
    }

    pub fn install_from_manifest(&self, manifest: OmniManifest) {
        for app in manifest.apps {
            match app.box_type.as_str() {
                "apt" => {
                    apt::install_with_apt(&app.name);
                    history::save_install(&app.name, "apt");
                }
                "pacman" => {
                    pacman::install_with_pacman(&app.name);
                    history::save_install(&app.name, "pacman");
                }
                "flatpak" => {
                    let name = app.source.as_deref().unwrap_or(&app.name);
                    flatpak::install_with_flatpak(name);
                    history::save_install(name, "flatpak");
                }
                _ => eprintln!("❌ Unknown box type: {}", app.box_type),
            }
        }
    }

    pub fn undo_last(&self) {
        history::undo_last_install();
    }

    pub fn snapshot(&self) {
        println!("📸 Creating a snapshot... [not implemented yet]");
    }

    pub fn revert(&self) {
        println!("⏪ Reverting to snapshot... [not implemented yet]");
    }
}
