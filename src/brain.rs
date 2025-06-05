use crate::boxes::{apt, dnf, flatpak, pacman};
use crate::distro;
use crate::history;
use crate::manifest::OmniManifest;

pub struct OmniBrain;

impl OmniBrain {
    pub fn new() -> Self {
        OmniBrain
    }

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
            "dnf" => {
                dnf::install_with_dnf(app);
                history::save_install(app, "dnf");
            }
            other => {
                eprintln!("❌ Unsupported distro: {}", other);
            }
        }
    }

    pub fn install_from_manifest(&self, manifest: OmniManifest) {
        let fallback = manifest
            .meta
            .as_ref()
            .and_then(|m| m.distro_fallback)
            .unwrap_or(false);

        for app in manifest.apps {
            let handled = match app.box_type.as_str() {
                "apt" if distro::command_exists("apt") => {
                    apt::install_with_apt(&app.name);
                    history::save_install(&app.name, "apt");
                    true
                }
                "pacman" if distro::command_exists("pacman") => {
                    pacman::install_with_pacman(&app.name);
                    history::save_install(&app.name, "pacman");
                    true
                }
                "dnf" if distro::command_exists("dnf") => {
                    dnf::install_with_dnf(&app.name);
                    history::save_install(&app.name, "dnf");
                    true
                }
                "flatpak" if distro::command_exists("flatpak") => {
                    let name = app.source.as_deref().unwrap_or(&app.name);
                    flatpak::install_with_flatpak(name);
                    history::save_install(name, "flatpak");
                    true
                }
                _ => false,
            };

            if !handled {
                if fallback {
                    match distro::detect_distro().as_str() {
                        "apt" if distro::command_exists("apt") => {
                            apt::install_with_apt(&app.name);
                            history::save_install(&app.name, "apt");
                        }
                        "pacman" if distro::command_exists("pacman") => {
                            pacman::install_with_pacman(&app.name);
                            history::save_install(&app.name, "pacman");
                        }
                        "dnf" if distro::command_exists("dnf") => {
                            dnf::install_with_dnf(&app.name);
                            history::save_install(&app.name, "dnf");
                        }
                        other => eprintln!("❌ Unsupported distro: {}", other),
                    }
                } else {
                    eprintln!(
                        "❌ Box '{}' not available and fallback disabled for {}",
                        app.box_type, app.name
                    );
                }
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
