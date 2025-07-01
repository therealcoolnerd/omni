use anyhow::Result;
use dirs;
use reqwest;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tracing::info;

const APPIMAGE_DIR: &str = ".local/share/applications/appimages";

pub async fn install_appimage(url: &str, name: &str) -> Result<()> {
    info!("Installing AppImage {} from {}", name, url);

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let appimage_dir = home_dir.join(APPIMAGE_DIR);

    // Create AppImage directory if it doesn't exist
    fs::create_dir_all(&appimage_dir)?;

    let filename = format!("{}.AppImage", name);
    let target_path = appimage_dir.join(&filename);

    // Download the AppImage
    info!("Downloading AppImage from {}", url);
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Write to file
    fs::write(&target_path, &bytes)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }

    info!(
        "✅ Successfully installed AppImage {} to {:?}",
        name, target_path
    );

    // Create desktop entry
    create_desktop_entry(name, &target_path)?;

    Ok(())
}

fn create_desktop_entry(name: &str, appimage_path: &Path) -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let desktop_dir = home_dir.join(".local/share/applications");

    fs::create_dir_all(&desktop_dir)?;

    let desktop_file = desktop_dir.join(format!("{}.desktop", name.to_lowercase()));
    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec={}\n\
         Icon=application-x-executable\n\
         Categories=Utility;\n\
         Terminal=false\n",
        name,
        appimage_path.display()
    );

    fs::write(&desktop_file, desktop_content)?;
    info!("Created desktop entry at {:?}", desktop_file);

    Ok(())
}

pub fn list_appimages() -> Result<Vec<String>> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let appimage_dir = home_dir.join(APPIMAGE_DIR);

    if !appimage_dir.exists() {
        return Ok(vec![]);
    }

    let entries = fs::read_dir(appimage_dir)?;
    let mut appimages = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("AppImage") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                appimages.push(name.to_string());
            }
        }
    }

    Ok(appimages)
}

pub fn remove_appimage(name: &str) -> Result<()> {
    info!("Removing AppImage: {}", name);

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let appimage_dir = home_dir.join(APPIMAGE_DIR);
    let appimage_path = appimage_dir.join(format!("{}.AppImage", name));

    if appimage_path.exists() {
        fs::remove_file(&appimage_path)?;
        info!("Removed AppImage file: {:?}", appimage_path);
    }

    // Remove desktop entry
    let desktop_dir = home_dir.join(".local/share/applications");
    let desktop_file = desktop_dir.join(format!("{}.desktop", name.to_lowercase()));

    if desktop_file.exists() {
        fs::remove_file(&desktop_file)?;
        info!("Removed desktop entry: {:?}", desktop_file);
    }

    info!("✅ Successfully removed AppImage {}", name);
    Ok(())
}

pub fn verify_appimage_hash(path: &Path, expected_hash: &str) -> Result<bool> {
    let contents = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let hash = hex::encode(hasher.finalize());

    Ok(hash.eq_ignore_ascii_case(expected_hash))
}

pub fn get_appimage_info(name: &str) -> Result<String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let appimage_dir = home_dir.join(APPIMAGE_DIR);
    let appimage_path = appimage_dir.join(format!("{}.AppImage", name));

    if !appimage_path.exists() {
        return Err(anyhow::anyhow!("AppImage {} not found", name));
    }

    let metadata = fs::metadata(&appimage_path)?;
    let size = metadata.len();
    let modified = metadata.modified()?;

    Ok(format!(
        "AppImage: {}\nPath: {}\nSize: {} bytes\nModified: {:?}",
        name,
        appimage_path.display(),
        size,
        modified
    ))
}
