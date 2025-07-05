mod advanced_resolver;
mod audit;
mod boxes;
mod brain;
mod config;
mod database;
mod distro;
mod docker;
mod error_handling;
#[cfg(feature = "gui")]
mod gui;
mod hardware;
mod history;
mod input_validation;
mod interactive;
mod logging;
mod manifest;
mod privilege_manager;
mod resolver;
mod runtime;
mod sandboxing;
mod search;
mod secure_brain;
mod secure_executor;
mod security;
mod snapshot;
#[cfg(feature = "ssh")]
mod ssh;
mod transaction;
mod types;
mod unified_manager;
mod updater;

use anyhow::Result;
use brain::OmniBrain;
use clap::{Parser, Subcommand};
use config::OmniConfig;
use manifest::OmniManifest;
use search::SearchEngine;
use snapshot::SnapshotManager;
use tracing::{error, info, warn};
use updater::UpdateManager;

#[derive(Parser)]
#[command(name = "omni")]
#[command(about = "Universal Cross-Platform Package Manager - Linux, Windows, macOS")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    mock: bool,

    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        /// Package name or --from manifest
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,

        /// Install from manifest file
        #[arg(long)]
        from: Option<String>,

        /// Specify package box type
        #[arg(long)]
        box_type: Option<String>,

        /// AppImage source URL
        #[arg(long)]
        url: Option<String>,
    },

    /// Remove/uninstall packages
    Remove {
        /// Package name
        package: String,

        /// Specify package box type
        #[arg(long)]
        box_type: Option<String>,
    },

    /// Search for packages across all sources
    Search {
        /// Search query
        query: String,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show package information
    Info {
        /// Package name
        package: String,

        /// Specify package box type
        #[arg(long)]
        box_type: Option<String>,
    },

    /// Update packages
    Update {
        /// Update specific package
        package: Option<String>,

        /// Update all packages
        #[arg(long)]
        all: bool,

        /// Refresh repositories first
        #[arg(long)]
        refresh: bool,
    },

    /// List installed packages
    List {
        /// Show only packages from specific box
        #[arg(long)]
        box_type: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Package history and rollback
    History {
        #[command(subcommand)]
        action: HistoryCommands,
    },

    /// Snapshot management
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },

    /// Launch GUI
    Gui,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Dependency resolution
    Resolve {
        /// Package name to resolve
        package: String,

        /// Specify package box type
        #[arg(long)]
        box_type: Option<String>,

        /// Show detailed resolution plan
        #[arg(short, long)]
        detailed: bool,
    },

    /// Security verification
    Verify {
        /// File path to verify
        file_path: String,

        /// Expected checksum
        #[arg(long)]
        checksum: Option<String>,

        /// Signature URL or file path
        #[arg(long)]
        signature: Option<String>,

        /// Package type
        #[arg(long)]
        box_type: Option<String>,
    },

    /// Hardware detection and driver management
    Hardware {
        #[command(subcommand)]
        action: HardwareCommands,
    },
}

#[derive(Subcommand)]
enum HistoryCommands {
    /// Show installation history
    Show {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: i64,
    },

    /// Undo last installation
    Undo,
}

#[derive(Subcommand)]
enum SnapshotCommands {
    /// Create a snapshot
    Create {
        /// Snapshot name
        name: String,

        /// Snapshot description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all snapshots
    List,

    /// Revert to a snapshot
    Revert {
        /// Snapshot ID or name
        snapshot: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Edit configuration
    Edit,

    /// Reset to defaults
    Reset,
}

#[derive(Subcommand)]
enum HardwareCommands {
    /// Detect server hardware and show information
    Detect,
    
    /// Auto-detect and install recommended drivers
    Install,
    
    /// Install drivers for specific vendor (Dell, HP, Supermicro, etc.)
    Vendor {
        /// Hardware vendor name
        vendor: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = OmniConfig::load()?;

    // Initialize logging
    logging::init_logging(&config)?;

    // Handle the command
    match handle_command(cli, config).await {
        Ok(_) => {}
        Err(e) => {
            error!("Application error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_command(cli: Cli, config: OmniConfig) -> Result<()> {
    match cli.command {
        Commands::Install {
            package,
            from,
            box_type,
            url,
        } => {
            let mut brain = OmniBrain::new_with_mock(cli.mock);

            if let Some(manifest_path) = from {
                let manifest = OmniManifest::from_file(&manifest_path)?;
                brain.install_from_manifest(manifest).await?;
            } else if let Some(package_name) = package {
                if let Some(url) = url {
                    // AppImage installation
                    if cli.mock {
                        println!(
                            "🎭 [MOCK] Would install AppImage {} from {}",
                            package_name, url
                        );
                    } else {
                        boxes::appimage::install_appimage(&url, &package_name).await?;
                        println!("✅ Successfully installed AppImage {}", package_name);
                    }
                } else {
                    brain.install(&package_name, box_type.as_deref()).await?;
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Please specify a package name or manifest file"
                ));
            }
        }

        Commands::Remove { package, box_type } => {
            let mut brain = OmniBrain::new_with_mock(cli.mock);
            brain.remove(&package, box_type.as_deref()).await?;
        }

        Commands::Search { query, limit } => {
            let search_engine = SearchEngine::new().await?;
            let results = search_engine.search_all(&query).await?;

            println!("🔍 Search results for '{}':\n", query);

            for (i, result) in results.iter().take(limit).enumerate() {
                let status = if result.installed {
                    "✅ Installed"
                } else {
                    "  Available"
                };
                println!(
                    "{}. {} [{}] {}",
                    i + 1,
                    result.name,
                    result.box_type,
                    status
                );

                if let Some(desc) = &result.description {
                    println!("   {}", desc);
                }
                println!();
            }

            if results.len() > limit {
                println!("... and {} more results", results.len() - limit);
            }
        }

        Commands::Info { package, box_type } => {
            let search_engine = SearchEngine::new().await?;

            if let Some(box_type) = box_type {
                if let Some(info) = search_engine.get_package_info(&package, &box_type).await? {
                    println!("{}", info);
                } else {
                    println!("❌ Package information not found");
                }
            } else {
                // Try all available box types
                let box_types = ["apt", "dnf", "pacman", "snap", "flatpak"];
                let mut found = false;

                for bt in &box_types {
                    if distro::command_exists(bt) {
                        if let Some(info) = search_engine.get_package_info(&package, bt).await? {
                            println!("📦 Information from {} box:\n{}\n", bt, info);
                            found = true;
                        }
                    }
                }

                if !found {
                    println!("❌ Package information not found in any available box");
                }
            }
        }

        Commands::Update {
            package,
            all,
            refresh,
        } => {
            let update_manager = UpdateManager::new(config).await?;

            if refresh {
                update_manager.refresh_repositories().await?;
            }

            if all {
                update_manager.update_all().await?;
            } else if let Some(package_name) = package {
                let candidates = update_manager.check_updates().await?;
                if let Some(candidate) = candidates.iter().find(|c| c.package_name == package_name)
                {
                    update_manager.update_package(candidate).await?;
                } else {
                    println!("✅ Package {} is already up to date", package_name);
                }
            } else {
                let candidates = update_manager.check_updates().await?;

                if candidates.is_empty() {
                    println!("✅ All packages are up to date");
                } else {
                    println!("📦 Available updates:");
                    for candidate in &candidates {
                        println!(
                            "  {} [{}]: {} -> {}",
                            candidate.package_name,
                            candidate.box_type,
                            candidate.current_version.as_deref().unwrap_or("unknown"),
                            candidate.available_version.as_deref().unwrap_or("latest")
                        );
                    }
                    println!("\nRun 'omni update --all' to update all packages");
                }
            }
        }

        Commands::List { box_type, detailed } => {
            let update_manager = UpdateManager::new(config).await?;
            let installed = update_manager.list_installed().await?;

            let filtered: Vec<_> = if let Some(bt) = box_type {
                installed.into_iter().filter(|p| p.box_type == bt).collect()
            } else {
                installed
            };

            if filtered.is_empty() {
                println!("No installed packages found");
                return Ok(());
            }

            println!("📦 Installed packages ({}):\n", filtered.len());

            for package in filtered {
                if detailed {
                    println!("Name: {}", package.package_name);
                    println!("Box: {}", package.box_type);
                    println!(
                        "Version: {}",
                        package.version.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "Installed: {}",
                        package.installed_at.format("%Y-%m-%d %H:%M:%S")
                    );
                    if let Some(source) = &package.source_url {
                        println!("Source: {}", source);
                    }
                    println!();
                } else {
                    println!(
                        "{} [{}] ({})",
                        package.package_name,
                        package.box_type,
                        package.version.as_deref().unwrap_or("unknown")
                    );
                }
            }
        }

        Commands::History { action } => match action {
            HistoryCommands::Show { limit } => {
                let db = database::Database::new().await?;
                let history = db.get_install_history(Some(limit)).await?;

                if history.is_empty() {
                    println!("No installation history found");
                    return Ok(());
                }

                println!("📜 Installation history:\n");

                for record in history {
                    let status = match record.status {
                        database::InstallStatus::Success => "✅ Installed",
                        database::InstallStatus::Updated => "🔄 Updated",
                        database::InstallStatus::Removed => "❌ Removed",
                        database::InstallStatus::Failed => "💥 Failed",
                    };

                    println!(
                        "{} {} [{}] - {}",
                        record.installed_at.format("%Y-%m-%d %H:%M:%S"),
                        record.package_name,
                        record.box_type,
                        status
                    );
                }
            }

            HistoryCommands::Undo => {
                let mut brain = OmniBrain::new_with_mock(cli.mock);
                brain.undo_last().await?;
            }
        },

        Commands::Snapshot { action } => {
            let snapshot_manager = SnapshotManager::new().await?;

            match action {
                SnapshotCommands::Create { name, description } => {
                    let snapshot_id = snapshot_manager
                        .create_snapshot(&name, description.as_deref())
                        .await?;
                    println!("✅ Created snapshot '{}' with ID: {}", name, snapshot_id);
                }

                SnapshotCommands::List => {
                    let snapshots = snapshot_manager.list_snapshots().await?;

                    if snapshots.is_empty() {
                        println!("No snapshots found");
                        return Ok(());
                    }

                    println!("📸 Available snapshots:\n");

                    for snapshot in snapshots {
                        println!("Name: {}", snapshot.name);
                        println!("ID: {}", snapshot.id);
                        println!(
                            "Created: {}",
                            snapshot.created_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!("Packages: {}", snapshot.packages.len());
                        if let Some(desc) = &snapshot.description {
                            println!("Description: {}", desc);
                        }
                        println!();
                    }
                }

                SnapshotCommands::Revert { snapshot } => {
                    snapshot_manager.revert_to_snapshot(&snapshot).await?;
                }
            }
        }

        Commands::Gui => {
            #[cfg(feature = "gui")]
            {
                gui::launch_gui();
            }
            #[cfg(not(feature = "gui"))]
            {
                println!("❌ GUI feature not compiled. Please rebuild with --features gui");
            }
        }

        Commands::Config { action } => {
            match action {
                ConfigCommands::Show => {
                    println!("📋 Current configuration:\n");
                    println!("{}", serde_yaml::to_string(&config)?);
                }

                ConfigCommands::Edit => {
                    let config_path = OmniConfig::config_path()?;
                    println!("📝 Edit configuration file: {}", config_path.display());

                    // Try to open with default editor
                    if let Ok(editor) = std::env::var("EDITOR") {
                        std::process::Command::new(editor)
                            .arg(&config_path)
                            .status()?;
                    } else {
                        println!("Set EDITOR environment variable or edit manually");
                    }
                }

                ConfigCommands::Reset => {
                    let default_config = OmniConfig::default();
                    default_config.save()?;
                    println!("✅ Configuration reset to defaults");
                }
            }
        }

        Commands::Resolve {
            package,
            box_type,
            detailed,
        } => {
            let resolver = resolver::DependencyResolver::new().await?;
            let plan = resolver
                .resolve_dependencies(&package, box_type.as_deref())
                .await?;

            println!("🔍 Dependency resolution for '{}':\n", package);

            if plan.packages.is_empty() {
                println!("No dependencies found or package not available.");
                return Ok(());
            }

            println!("📦 Packages to install ({}):", plan.packages.len());
            for (i, pkg) in plan.packages.iter().enumerate() {
                let marker = if i == 0 { "🎯" } else { "📎" };
                println!(
                    "{} {} [{}] v{}",
                    marker, pkg.name, pkg.box_type, pkg.version
                );

                if detailed && !pkg.dependencies.is_empty() {
                    for dep in &pkg.dependencies {
                        let opt = if dep.optional { " (optional)" } else { "" };
                        println!("   └─ {}{}", dep.name, opt);
                    }
                }
            }

            if let Some(size) = plan.total_size {
                println!(
                    "\n💾 Total size: {}",
                    resolver::DependencyResolver::format_size(size)
                );
            }

            if !plan.conflicts.is_empty() {
                println!("\n⚠️  Conflicts:");
                for conflict in &plan.conflicts {
                    println!("   • {}", conflict);
                }
            }

            if !plan.warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for warning in &plan.warnings {
                    println!("   • {}", warning);
                }
            }
        }

        Commands::Verify {
            file_path,
            checksum,
            signature,
            box_type,
        } => {
            use security::{SecurityPolicy, SecurityVerifier};
            use std::path::Path;

            let policy = SecurityPolicy::default();
            let verifier = SecurityVerifier::new(policy);

            let path = Path::new(&file_path);
            if !path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file_path));
            }

            println!("🔒 Verifying security for: {}", file_path);

            let result = verifier
                .verify_package(
                    path,
                    checksum.as_deref(),
                    signature.as_deref(),
                    box_type.as_deref().unwrap_or("unknown"),
                )
                .await?;

            println!("\n📋 Verification Results:");
            println!("{}", "─".repeat(50));
            println!("{}", result.details);

            match result.trust_level {
                security::TrustLevel::Trusted => println!("✅ Package is trusted and verified"),
                security::TrustLevel::Valid => println!("✅ Package signature is valid"),
                security::TrustLevel::Unsigned => {
                    println!("⚠️  Package is unsigned but checksum verified")
                }
                security::TrustLevel::Untrusted => println!("❌ Package failed verification"),
            }
        }

        Commands::Hardware { action } => {
            let mut brain = OmniBrain::new_with_mock(cli.mock);
            
            match action {
                HardwareCommands::Detect => {
                    println!("🔍 Detecting server hardware configuration...");
                    match brain.get_hardware_info() {
                        Ok(info) => {
                            println!("\n📋 Hardware Information:");
                            println!("{}", "─".repeat(50));
                            println!("{}", info);
                        }
                        Err(e) => {
                            error!("❌ Failed to detect hardware: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                HardwareCommands::Install => {
                    println!("🤖 Auto-detecting hardware and installing optimal drivers...");
                    match brain.detect_and_install_drivers().await {
                        Ok(()) => {
                            println!("✅ Driver installation process completed");
                        }
                        Err(e) => {
                            error!("❌ Driver installation failed: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                HardwareCommands::Vendor { vendor } => {
                    println!("🏢 Installing {} vendor-specific drivers...", vendor);
                    match brain.install_vendor_drivers(&vendor).await {
                        Ok(()) => {
                            println!("✅ {} vendor drivers installation completed", vendor);
                        }
                        Err(e) => {
                            error!("❌ Vendor driver installation failed: {}", e);
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
