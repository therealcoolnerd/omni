use crate::brain::OmniBrain;
use crate::manifest::OmniManifest;
use eframe::{egui, App};

pub struct OmniGui {
    brain: OmniBrain,
    package_input: String,
    manifest_path: String,
}

impl Default for OmniGui {
    fn default() -> Self {
        Self {
            brain: OmniBrain::new(),
            package_input: String::new(),
            manifest_path: String::new(),
        }
    }
}

impl App for OmniGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Omni Flame");
            ui.horizontal(|ui| {
                ui.label("Package:");
                ui.text_edit_singleline(&mut self.package_input);
                if ui.button("Install").clicked() {
                    if !self.package_input.is_empty() {
                        self.brain.install(&self.package_input);
                        self.package_input.clear();
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Manifest:");
                ui.text_edit_singleline(&mut self.manifest_path);
                if ui.button("Install Manifest").clicked() {
                    if let Ok(manifest) = OmniManifest::from_file(&self.manifest_path) {
                        self.brain.install_from_manifest(manifest);
                    }
                }
            });
            ui.separator();
            if ui.button("Undo Last").clicked() {
                self.brain.undo_last();
            }
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
