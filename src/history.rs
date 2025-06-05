
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufReader;
use crate::boxes::{apt, dnf, flatpak, pacman};

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
            "apt" => apt::uninstall_with_apt(&last.package),
            "pacman" => pacman::uninstall_with_pacman(&last.package),
            "dnf" => dnf::uninstall_with_dnf(&last.package),
            "flatpak" => flatpak::uninstall_with_flatpak(&last.package),
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
