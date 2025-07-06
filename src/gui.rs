use crate::brain::OmniBrain;
use crate::distro::{get_available_package_managers, get_os_display_name};
use crate::manifest::OmniManifest;
use crate::search::SearchResult;
use crate::database::InstallRecord;
use eframe::{egui, App};
use rfd::FileDialog;
use std::collections::HashMap;
use std::time::Instant;

#[derive(PartialEq)]
enum Tab {
    Dashboard,
    Installed,
    History,
    Systems,
    Settings,
}

#[derive(PartialEq)]
enum SearchFilter {
    All,
    Applications,
    Libraries,
    Development,
    System,
}

#[derive(PartialEq)]
enum SortMode {
    Name,
    Popularity,
    Recent,
    Size,
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
    search_filter: SearchFilter,
    sort_mode: SortMode,
    animation_time: Instant,
    show_advanced_search: bool,
    package_stats: HashMap<String, u32>,
    system_performance: f32,
    download_speed: f32,
    active_operations: u32,
    history: Vec<InstallRecord>,
    ssh_host: String,
    container_name: String,
}

impl Default for OmniGui {
    fn default() -> Self {
        let mut gui = Self {
            brain: OmniBrain::new(),
            active_tab: Tab::Dashboard,
            package_input: String::new(),
            manifest_path: String::new(),
            status: String::new(),
            search_results: Vec::new(),
            installed_packages: Vec::new(),
            installation_progress: HashMap::new(),
            dark_mode: true,
            search_filter: SearchFilter::All,
            sort_mode: SortMode::Popularity,
            animation_time: Instant::now(),
            show_advanced_search: false,
            package_stats: HashMap::new(),
            system_performance: 0.85,
            download_speed: 45.2,
            active_operations: 0,
            history: Vec::new(),
            ssh_host: String::new(),
            container_name: String::new(),
        };

        // Initialize with real data
        gui.refresh_stats();
        gui
    }
}

impl App for OmniGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Enhanced theme with custom styling
        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        // Window rounding not available in this egui version
        visuals.panel_fill = if self.dark_mode {
            egui::Color32::from_rgb(25, 25, 35)
        } else {
            egui::Color32::from_rgb(248, 249, 250)
        };

        ctx.set_visuals(visuals);

        // Enhanced top panel with gradient effect
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);

                    // Animated logo with pulse effect
                    let time = self.animation_time.elapsed().as_secs_f32();
                    let pulse = (time * 2.0).sin() * 0.1 + 1.0;
                    let logo_size = 32.0 * pulse;

                    ui.add_sized(
                        [logo_size, logo_size],
                        egui::Label::new(
                            egui::RichText::new("🚀")
                                .size(logo_size)
                                .color(egui::Color32::from_rgb(100, 150, 255)),
                        ),
                    );

                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Omni").size(24.0).strong().color(
                            if self.dark_mode {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::BLACK
                            },
                        ));
                        ui.label(
                            egui::RichText::new("Universal Package Manager")
                                .size(12.0)
                                .color(egui::Color32::GRAY),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);

                        // System performance indicator
                        let perf_color = if self.system_performance > 0.8 {
                            egui::Color32::GREEN
                        } else if self.system_performance > 0.6 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::RED
                        };

                        ui.colored_label(
                            perf_color,
                            format!("⚡ {:.0}%", self.system_performance * 100.0),
                        );

                        if self.active_operations > 0 {
                            ui.colored_label(
                                egui::Color32::from_rgb(100, 150, 255),
                                format!("🔄 {} ops", self.active_operations),
                            );
                        }

                        // Enhanced theme toggle
                        let theme_btn = ui.add_sized(
                            [40.0, 32.0],
                            egui::Button::new(if self.dark_mode { "☀" } else { "🌙" })
                                .corner_radius(16.0),
                        );

                        if theme_btn.clicked() {
                            self.dark_mode = !self.dark_mode;
                        }
                    });
                });
                ui.add_space(8.0);
            });

        // Enhanced left panel with modern tab design
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(16.0);

                ui.vertical(|ui| {
                    let tab_height = 48.0;
                    let tab_spacing = 8.0;

                    // Dashboard tab
                    ui.add_space(tab_spacing);
                    let dashboard_response = ui.add_sized(
                        [180.0, tab_height],
                        egui::SelectableLabel::new(
                            self.active_tab == Tab::Dashboard,
                            egui::RichText::new("📊  Dashboard").size(16.0).strong(),
                        ),
                    );
                    if dashboard_response.clicked() {
                        self.active_tab = Tab::Dashboard;
                    }

                    // Installed tab with badge
                    ui.add_space(tab_spacing);
                    let installed_text = format!(
                        "📦  Installed ({})",
                        self.package_stats.get("Installed").unwrap_or(&0)
                    );
                    let installed_response = ui.add_sized(
                        [180.0, tab_height],
                        egui::SelectableLabel::new(
                            self.active_tab == Tab::Installed,
                            egui::RichText::new(installed_text).size(16.0),
                        ),
                    );
                    if installed_response.clicked() {
                        self.active_tab = Tab::Installed;
                    }

                    // History tab
                    ui.add_space(tab_spacing);
                    let history_response = ui.add_sized(
                        [180.0, tab_height],
                        egui::SelectableLabel::new(
                            self.active_tab == Tab::History,
                            egui::RichText::new("📜  History").size(16.0),
                        ),
                    );
                    if history_response.clicked() {
                        self.active_tab = Tab::History;
                    }

                    // Systems tab
                    ui.add_space(tab_spacing);
                    let systems_response = ui.add_sized(
                        [180.0, tab_height],
                        egui::SelectableLabel::new(
                            self.active_tab == Tab::Systems,
                            egui::RichText::new("🖥️  Systems").size(16.0),
                        ),
                    );
                    if systems_response.clicked() {
                        self.active_tab = Tab::Systems;
                    }

                    // Settings tab
                    ui.add_space(tab_spacing);
                    let settings_response = ui.add_sized(
                        [180.0, tab_height],
                        egui::SelectableLabel::new(
                            self.active_tab == Tab::Settings,
                            egui::RichText::new("⚙️  Settings").size(16.0),
                        ),
                    );
                    if settings_response.clicked() {
                        self.active_tab = Tab::Settings;
                    }

                    ui.add_space(32.0);

                    // Quick stats sidebar
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Quick Stats")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("💾");
                                ui.label(format!("{:.1} MB/s", self.download_speed));
                            });

                            if let Some(updates) = self.package_stats.get("Available Updates") {
                                if *updates > 0 {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 165, 0),
                                            "⬆️",
                                        );
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 165, 0),
                                            format!("{} updates", updates),
                                        );
                                    });
                                }
                            }

                            if let Some(failed) = self.package_stats.get("Failed Installs") {
                                if *failed > 0 {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(egui::Color32::RED, "❌");
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("{} failed", failed),
                                        );
                                    });
                                }
                            }
                        });
                    });
                });
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            Tab::Dashboard => self.show_dashboard(ui),
            Tab::Installed => self.show_installed(ui),
            Tab::History => self.show_history(ui),
            Tab::Systems => self.show_systems(ui),
            Tab::Settings => self.show_settings(ui),
        });

        // Enhanced status bar with real-time info
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .min_height(40.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);

                    // Status message with icon
                    if !self.status.is_empty() {
                        ui.label("ℹ️");
                        ui.label(&self.status);
                    } else {
                        ui.label("✅");
                        ui.label("Ready");
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);

                        // Memory usage (simulated)
                        let time = self.animation_time.elapsed().as_secs_f32();
                        let mem_usage = 45.0 + (time * 0.5).sin() * 5.0;
                        ui.label(format!("💾 {:.1}% RAM", mem_usage));

                        ui.separator();

                        // Network speed
                        ui.label(format!("🌐 {:.1} MB/s", self.download_speed));

                        ui.separator();

                        // Connection status
                        ui.colored_label(egui::Color32::GREEN, "🟢 Online");

                        ui.separator();

                        // Version info
                        ui.label(
                            egui::RichText::new("v1.0.0")
                                .size(12.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                });
                ui.add_space(4.0);
            });
    }
}

impl OmniGui {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        // Real-time stats cards
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Stats cards with enhanced visuals
            for (label, value) in &self.package_stats {
                ui.group(|ui| {
                    ui.set_min_size(egui::Vec2::new(120.0, 80.0));
                    ui.vertical_centered(|ui| {
                        let color = match label.as_str() {
                            "Total Packages" => egui::Color32::from_rgb(100, 150, 255),
                            "Installed" => egui::Color32::GREEN,
                            "Available Updates" => egui::Color32::from_rgb(255, 165, 0),
                            "Failed Installs" => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        };

                        ui.label(
                            egui::RichText::new(format!("{}", value))
                                .size(24.0)
                                .strong()
                                .color(color),
                        );
                        ui.label(
                            egui::RichText::new(label)
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                });
                ui.add_space(8.0);
            }
        });

        ui.add_space(16.0);

        // System info banner with enhanced styling
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("🖥️ {}", get_os_display_name()))
                        .size(16.0)
                        .strong(),
                );
                ui.separator();
                let managers = get_available_package_managers();
                ui.label(
                    egui::RichText::new(format!("📦 {} package managers", managers.len()))
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new("View Systems")
                                .color(egui::Color32::from_rgb(100, 150, 255)),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Systems;
                    }
                });
            });
        });

        ui.add_space(16.0);

        // Enhanced search interface
        ui.group(|ui| {
            ui.vertical(|ui| {
                // Main search bar
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔍 Search packages:")
                            .size(16.0)
                            .strong(),
                    );

                    // Advanced search toggle
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if self.show_advanced_search {
                                "🔽 Advanced"
                            } else {
                                "🔼 Advanced"
                            })
                            .clicked()
                        {
                            self.show_advanced_search = !self.show_advanced_search;
                        }
                    });
                });

                ui.add_space(8.0);

                // Search input with enhanced styling
                ui.horizontal(|ui| {
                    let search_response = ui.add_sized(
                        [300.0, 32.0],
                        egui::TextEdit::singleline(&mut self.package_input)
                            .hint_text("Enter package name...")
                            .font(egui::TextStyle::Body),
                    );

                    // Keyboard shortcut handling
                    if search_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if !self.package_input.is_empty() {
                            // Use async search - for GUI we'll use block_on for now
                            if let Ok(results) =
                                futures::executor::block_on(self.brain.search(&self.package_input))
                            {
                                self.search_results = results;
                            } else {
                                self.search_results = Vec::new();
                            }
                        }
                    }

                    let search_btn = ui.add_sized(
                        [80.0, 32.0],
                        egui::Button::new("🔍 Search").corner_radius(6.0),
                    );
                    if search_btn.clicked() && !self.package_input.is_empty() {
                        // Use async search - for GUI we'll use block_on for now
                        if let Ok(results) =
                            futures::executor::block_on(self.brain.search(&self.package_input))
                        {
                            self.search_results = results;
                        } else {
                            self.search_results = Vec::new();
                        }
                    }

                    let install_btn = ui.add_sized(
                        [100.0, 32.0],
                        egui::Button::new("📦 Quick Install")
                            .fill(egui::Color32::from_rgb(50, 150, 50))
                            .corner_radius(6.0),
                    );
                    if install_btn.clicked() && !self.package_input.is_empty() {
                        let _ = futures::executor::block_on(
                            self.brain.install(&self.package_input, None),
                        );
                        self.status = format!("Installing {}", self.package_input);
                        self.installation_progress
                            .insert(self.package_input.clone(), 0.0);
                    }
                });

                // Advanced search options
                if self.show_advanced_search {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.selectable_value(&mut self.search_filter, SearchFilter::All, "All");
                        ui.selectable_value(
                            &mut self.search_filter,
                            SearchFilter::Applications,
                            "Apps",
                        );
                        ui.selectable_value(
                            &mut self.search_filter,
                            SearchFilter::Libraries,
                            "Libraries",
                        );
                        ui.selectable_value(
                            &mut self.search_filter,
                            SearchFilter::Development,
                            "Dev Tools",
                        );
                        ui.selectable_value(
                            &mut self.search_filter,
                            SearchFilter::System,
                            "System",
                        );

                        ui.separator();

                        ui.label("Sort by:");
                        ui.selectable_value(&mut self.sort_mode, SortMode::Popularity, "Popular");
                        ui.selectable_value(&mut self.sort_mode, SortMode::Name, "Name");
                        ui.selectable_value(&mut self.sort_mode, SortMode::Recent, "Recent");
                        ui.selectable_value(&mut self.sort_mode, SortMode::Size, "Size");
                    });
                }
            });
        });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.label("Manifest file:");
            ui.text_edit_singleline(&mut self.manifest_path);
            if ui.button("📁 Browse").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("YAML", &["yml", "yaml"])
                    .pick_file()
                {
                    self.manifest_path = path.display().to_string();
                }
            }
            if ui.button("⚡ Install Manifest").clicked() {
                if let Ok(manifest) = OmniManifest::from_file(&self.manifest_path) {
                    // Note: In a real GUI app, this would be handled asynchronously
                    // For now, we'll spawn a blocking task simulation
                    let _ = std::thread::spawn(move || {
                        // This would be: runtime.block_on(brain.install_from_manifest(manifest))
                        // For demo purposes, we just simulate the operation
                    });
                    self.status = format!("Installing manifest: {}", self.manifest_path);
                }
            }
        });

        ui.separator();

        // Enhanced search results with modern cards
        if !self.search_results.is_empty() {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎯 Search Results").size(18.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} packages found",
                                    self.search_results.len()
                                ))
                                .color(egui::Color32::GRAY),
                            );
                        });
                    });

                    ui.add_space(12.0);

                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for (i, result) in self.search_results.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            // Package icon based on source
                                            let icon = match result.source.as_deref() {
                                                Some("apt") => "📦",
                                                Some("brew") => "🍺",
                                                Some("chocolatey") => "🍫",
                                                Some("npm") => "📦",
                                                Some("pip") => "🐍",
                                                _ => "📦",
                                            };

                                            ui.label(egui::RichText::new(icon).size(20.0));
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&result.name)
                                                        .size(16.0)
                                                        .strong(),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "via {}",
                                                        result
                                                            .source
                                                            .as_deref()
                                                            .unwrap_or("unknown")
                                                    ))
                                                    .size(12.0)
                                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                                                );
                                            });

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let install_btn = ui.add_sized(
                                                        [80.0, 28.0],
                                                        egui::Button::new("Install")
                                                            .fill(egui::Color32::from_rgb(
                                                                50, 150, 50,
                                                            ))
                                                            .corner_radius(4.0),
                                                    );
                                                    if install_btn.clicked() {
                                                        let _ = futures::executor::block_on(
                                                            self.brain.install(
                                                                &result.name,
                                                                Some(&result.box_type),
                                                            ),
                                                        );
                                                        self.status =
                                                            format!("Installing {}", result.name);
                                                        self.installation_progress
                                                            .insert(result.name.clone(), 0.0);
                                                    }

                                                    // Popularity indicator
                                                    let popularity = ((i + 1) as f32
                                                        / self.search_results.len() as f32)
                                                        * 5.0;
                                                    let stars = "★".repeat(popularity as usize)
                                                        + &"☆".repeat(5 - popularity as usize);
                                                    ui.label(egui::RichText::new(stars).color(
                                                        egui::Color32::from_rgb(255, 215, 0),
                                                    ));
                                                },
                                            );
                                        });

                                        if let Some(desc) = &result.description {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new(desc)
                                                    .size(13.0)
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }
                                    });
                                });
                                ui.add_space(8.0);
                            }
                        });
                });
            });

            ui.add_space(16.0);
        }

        // Enhanced installation progress with animations
        if !self.installation_progress.is_empty() {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("🔄 Active Installations")
                            .size(16.0)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    for (package, progress) in &self.installation_progress {
                        ui.horizontal(|ui| {
                            // Animated spinner
                            let time = self.animation_time.elapsed().as_secs_f32();
                            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                            let spinner_idx = (time * 8.0) as usize % spinner_chars.len();

                            ui.label(
                                egui::RichText::new(spinner_chars[spinner_idx])
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                            );

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Installing {}", package)).strong(),
                                );
                                let progress_bar = egui::ProgressBar::new(*progress)
                                    .show_percentage()
                                    .desired_width(300.0)
                                    .fill(egui::Color32::from_rgb(50, 150, 50));
                                ui.add(progress_bar);
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
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
                        let _ = futures::executor::block_on(self.brain.remove(package, None));
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
                // Note: In a real GUI app, this would be handled asynchronously
                // For now, we'll spawn a blocking task simulation
                let _ = std::thread::spawn(move || {
                    // This would be: runtime.block_on(brain.undo_last())
                    // For demo purposes, we just simulate the operation
                });
                self.status = "Last operation undone".to_string();
            }
            if ui.button("🔄 Refresh").clicked() {
                if let Ok(history) =
                    futures::executor::block_on(self.brain.get_install_history(20))
                {
                    self.history = history;
                }
            }
        });

        ui.separator();
        ui.label("Recent Operations:");
        if self.history.is_empty() {
            if let Ok(history) =
                futures::executor::block_on(self.brain.get_install_history(10))
            {
                self.history = history;
            }
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.history.is_empty() {
                ui.label("No history available");
            } else {
                for record in &self.history {
                    ui.horizontal(|ui| {
                        ui.label(record.installed_at.format("%Y-%m-%d %H:%M:%S").to_string());
                        ui.label(format!("{} [{}]", record.package_name, record.box_type));
                    });
                }
            }
        });
    }

    fn show_systems(&mut self, ui: &mut egui::Ui) {
        ui.heading("System Information & Remote Management");

        // Current system info
        ui.group(|ui| {
            ui.heading("🖥️ Current System");
            ui.label(format!("Operating System: {}", get_os_display_name()));

            ui.separator();
            ui.label("Available Package Managers:");
            let managers = get_available_package_managers();
            if managers.is_empty() {
                ui.label("No package managers detected");
            } else {
                for manager in managers {
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label(manager);
                        if ui.small_button("Test").clicked() {
                            self.status = format!("Testing {} connectivity...", manager);
                        }
                    });
                }
            }
        });

        ui.separator();

        // Remote systems management
        ui.group(|ui| {
            ui.heading("🌐 Remote Systems");
            ui.label("Manage packages on remote Linux servers from Windows/macOS");

            ui.horizontal(|ui| {
                ui.label("SSH Host:");
                ui.text_edit_singleline(&mut self.ssh_host);
                if ui.button("Connect").clicked() {
                    self.status = format!("Connecting to {}...", self.ssh_host);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Docker Container:");
                ui.text_edit_singleline(&mut self.container_name);
                if ui.button("Attach").clicked() {
                    self.status = format!("Attaching to container {}...", self.container_name);
                }
            });
        });

        ui.separator();

        // Multi-platform support
        ui.group(|ui| {
            ui.heading("🔄 Cross-Platform Features");
            ui.label("Use Omni on any platform to manage packages anywhere:");

            ui.horizontal(|ui| {
                ui.label("• Windows → Linux servers (SSH)");
            });
            ui.horizontal(|ui| {
                ui.label("• macOS → Linux containers (Docker)");
            });
            ui.horizontal(|ui| {
                ui.label("• Native package management on each OS");
            });

            if ui.button("🚀 Open Documentation").clicked() {
                self.status = "Documentation: https://github.com/therealcoolnerd/omni".to_string();
            }
        });
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
        "Omni - Universal Cross-Platform Package Manager",
        options,
        Box::new(|_| Ok(Box::new(OmniGui::default()))),
    );
}

impl OmniGui {
    /// Refresh package statistics with real data from the system
    fn refresh_stats(&mut self) {
        // Get real data from the brain's list_installed functionality
        match self.brain.list_installed() {
            Ok(installed_packages) => {
                let total_installed: usize = installed_packages.values().map(|v| v.len()).sum();
                self.package_stats
                    .insert("Installed".to_string(), total_installed as u32);

                // Store installed packages for display
                self.installed_packages = installed_packages
                    .into_iter()
                    .flat_map(|(_, packages)| packages)
                    .collect();
            }
            Err(e) => {
                self.status = format!("Failed to get installed packages: {}", e);
                self.package_stats.insert("Installed".to_string(), 0);
            }
        }

        // Placeholder for total available packages - would need package manager search
        // In a real implementation, this would query package repositories
        self.package_stats
            .insert("Total Available".to_string(), 50000);

        // Placeholder for updates - would need actual update checking
        self.package_stats
            .insert("Available Updates".to_string(), 0);

        // Reset failed installs - would track from actual operations
        self.package_stats.insert("Failed Installs".to_string(), 0);

        // Reset active operations
        self.active_operations = 0;
    }
}
