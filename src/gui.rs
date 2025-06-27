use crate::brain::OmniBrain;
use crate::manifest::OmniManifest;
use crate::search::SearchResult;
use eframe::{egui, App};
use rfd::FileDialog;
use std::collections::HashMap;

#[derive(PartialEq)]
enum Tab {
    Dashboard,
    Installed,
    History,
    Settings,
}

pub struct OmniGui {
    brain: OmniBrain,
    active_tab: Tab,
    package_input: String,
    manifest_path: String,
    status: String,
    search_results: Vec<SearchResult>,
    installed_packages: Vec<String>,
    installation_progress: HashMap<String, f32>,
    dark_mode: bool,
}

impl Default for OmniGui {
    fn default() -> Self {
        Self {
            brain: OmniBrain::new(),
            active_tab: Tab::Dashboard,
            package_input: String::new(),
            manifest_path: String::new(),
            status: String::new(),
            search_results: Vec::new(),
            installed_packages: Vec::new(),
            installation_progress: HashMap::new(),
            dark_mode: true,
        }
    }
}

impl App for OmniGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Top panel with navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Omni Universal Package Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀" } else { "🌙" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });
        });

        // Left panel with tabs
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.active_tab, Tab::Installed, "📦 Installed");
                ui.selectable_value(&mut self.active_tab, Tab::History, "📜 History");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️ Settings");
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Dashboard => self.show_dashboard(ui),
                Tab::Installed => self.show_installed(ui),
                Tab::History => self.show_history(ui),
                Tab::Settings => self.show_settings(ui),
            }
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
            });
        });
    }
}

impl OmniGui {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Package Installation");
        
        ui.horizontal(|ui| {
            ui.label("Search packages:");
            ui.text_edit_singleline(&mut self.package_input);
            if ui.button("🔍 Search").clicked() {
                if !self.package_input.is_empty() {
                    self.search_results = self.brain.search(&self.package_input);
                }
            }
            if ui.button("📦 Install").clicked() {
                if !self.package_input.is_empty() {
                    self.brain.install(&self.package_input);
                    self.status = format!("Installing {}", self.package_input);
                    self.installation_progress.insert(self.package_input.clone(), 0.0);
                }
            }
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Manifest file:");
            ui.text_edit_singleline(&mut self.manifest_path);
            if ui.button("📁 Browse").clicked() {
                if let Some(path) = FileDialog::new().add_filter("YAML", &["yml", "yaml"]).pick_file() {
                    self.manifest_path = path.display().to_string();
                }
            }
            if ui.button("⚡ Install Manifest").clicked() {
                if let Ok(manifest) = OmniManifest::from_file(&self.manifest_path) {
                    self.brain.install_from_manifest(manifest);
                    self.status = format!("Installing manifest: {}", self.manifest_path);
                }
            }
        });
        
        ui.separator();
        
        // Show search results
        if !self.search_results.is_empty() {
            ui.heading("Search Results");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for result in &self.search_results {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&result.name);
                            ui.label(format!("[{}]", result.source));
                            if ui.button("Install").clicked() {
                                self.brain.install(&result.name);
                                self.status = format!("Installing {}", result.name);
                            }
                        });
                        if let Some(desc) = &result.description {
                            ui.label(desc);
                        }
                    });
                }
            });
        }
        
        // Show installation progress
        for (package, progress) in &self.installation_progress {
            ui.horizontal(|ui| {
                ui.label(format!("Installing {}", package));
                ui.add(egui::ProgressBar::new(*progress).show_percentage());
            });
        }
    }
    
    fn show_installed(&mut self, ui: &mut egui::Ui) {
        ui.heading("Installed Packages");
        
        if ui.button("🔄 Refresh List").clicked() {
            self.installed_packages = self.brain.list_installed();
        }
        
        if ui.button("⬆️ Update All").clicked() {
            self.brain.update_all();
            self.status = "Updating all packages...".to_string();
        }
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for package in &self.installed_packages {
                ui.horizontal(|ui| {
                    ui.label(package);
                    if ui.button("🗑️ Remove").clicked() {
                        self.brain.remove(package);
                        self.status = format!("Removing {}", package);
                    }
                });
            }
        });
    }
    
    fn show_history(&mut self, ui: &mut egui::Ui) {
        ui.heading("Installation History & Snapshots");
        
        ui.horizontal(|ui| {
            if ui.button("📸 Create Snapshot").clicked() {
                self.brain.create_snapshot();
                self.status = "Snapshot created".to_string();
            }
            if ui.button("↩️ Undo Last").clicked() {
                self.brain.undo_last();
                self.status = "Last operation undone".to_string();
            }
        });
        
        ui.separator();
        ui.label("Recent Operations:");
        // TODO: Show actual history from brain
    }
    
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        
        ui.checkbox(&mut self.dark_mode, "Dark Mode");
        
        ui.separator();
        ui.label("Package Manager Preferences:");
        // TODO: Add package manager priority settings
        
        ui.separator();
        if ui.button("🔄 Reset to Defaults").clicked() {
            self.status = "Settings reset to defaults".to_string();
        }
    }
}

pub fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "Omni Universal Package Manager",
        options,
        Box::new(|_| Ok(Box::new(OmniGui::default()))),
    );
}
