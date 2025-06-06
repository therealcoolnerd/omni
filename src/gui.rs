use crate::brain::OmniBrain;
use crate::manifest::OmniManifest;
use eframe::{egui, App};
use rfd::FileDialog;

pub struct OmniGui {
    brain: OmniBrain,
    package_input: String,
    manifest_path: String,
    status: String,
}

impl Default for OmniGui {
    fn default() -> Self {
        Self {
            brain: OmniBrain::new(),
            package_input: String::new(),
            manifest_path: String::new(),
            status: String::new(),
        }
    }
}

impl App for OmniGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Omni Flame");
            ui.label("Easy package installs for every distro");
            ui.horizontal(|ui| {
                ui.label("Package:");
                ui.text_edit_singleline(&mut self.package_input);
                if ui.button("Install").clicked() {
                    if !self.package_input.is_empty() {
                        self.brain.install(&self.package_input);
                        self.status = format!("Installed {}", self.package_input);
                        self.package_input.clear();
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Manifest:");
                ui.text_edit_singleline(&mut self.manifest_path);
                if ui.button("Browse").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.manifest_path = path.display().to_string();
                    }
                }
                if ui.button("Install Manifest").clicked() {
                    if let Ok(manifest) = OmniManifest::from_file(&self.manifest_path) {
                        self.brain.install_from_manifest(manifest);
                        self.status = format!("Installed manifest {}", self.manifest_path);
                    }
                }
            });
            ui.separator();
            if ui.button("Undo Last").clicked() {
                self.brain.undo_last();
                self.status = "Last install undone".to_string();
            }
            ui.separator();
            ui.label(&self.status);
        });
    }
}

pub fn launch_gui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Omni Flame",
        options,
        Box::new(|_| Ok(Box::new(OmniGui::default()))),
    );
}
