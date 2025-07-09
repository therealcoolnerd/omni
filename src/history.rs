use crate::boxes::apt::AptManager;
use crate::boxes::dnf::DnfBox;
use crate::boxes::flatpak::FlatpakBox;
use crate::boxes::pacman::PacmanBox;
use crate::distro::PackageManager;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufReader;

const HISTORY_FILE: &str = ".omni_history.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallRecord {
    pub package: String,
    pub box_type: String,
    pub timestamp: String,
}

pub fn save_install(package: &str, box_type: &str) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let record = InstallRecord {
        package: package.to_string(),
        box_type: box_type.to_string(),
        timestamp,
    };

    let mut history = load_history();
    history.push(record);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(HISTORY_FILE)
        .unwrap();

    serde_json::to_writer_pretty(file, &history).unwrap();
}

pub fn load_history() -> Vec<InstallRecord> {
    let file = OpenOptions::new().read(true).open(HISTORY_FILE);
    match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_) => vec![],
    }
}

pub fn undo_last_install() {
    let mut history = load_history();
    if let Some(last) = history.pop() {
        println!("🧹 Undoing '{}' via '{}'", last.package, last.box_type);
        match last.box_type.as_str() {
            "apt" => {
                if let Ok(apt_manager) = AptManager::new() {
                    if let Err(e) = apt_manager.remove(&last.package) {
                        eprintln!("❌ Failed to remove {} via apt: {}", last.package, e);
                    }
                } else {
                    eprintln!("❌ Failed to create apt manager");
                }
            }
            "pacman" => {
                if let Ok(pacman_manager) = PacmanBox::new() {
                    if let Err(e) = pacman_manager.remove(&last.package) {
                        eprintln!("❌ Failed to remove {} via pacman: {}", last.package, e);
                    }
                } else {
                    eprintln!("❌ Failed to create pacman manager");
                }
            }
            "dnf" => {
                if let Ok(dnf_manager) = DnfBox::new() {
                    if let Err(e) = dnf_manager.remove(&last.package) {
                        eprintln!("❌ Failed to remove {} via dnf: {}", last.package, e);
                    }
                } else {
                    eprintln!("❌ Failed to create dnf manager");
                }
            }
            "flatpak" => {
                if let Ok(flatpak_manager) = FlatpakBox::new() {
                    if let Err(e) = flatpak_manager.remove(&last.package) {
                        eprintln!("❌ Failed to remove {} via flatpak: {}", last.package, e);
                    }
                } else {
                    eprintln!("❌ Failed to create flatpak manager");
                }
            }
            other => eprintln!("❌ Unknown box type: {}", other),
        }
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(HISTORY_FILE)
            .unwrap();
        serde_json::to_writer_pretty(file, &history).unwrap();
    } else {
        println!("📭 No install history found.");
    }
}
