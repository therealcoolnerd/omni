use std::process::Command;
use anyhow::Result;
use tracing::{info, warn, error};

pub fn install_with_snap(app: &str) -> Result<()> {
    info!("Installing {} with snap", app);
    
    let output = Command::new("snap")
        .arg("install")
        .arg(app)
        .output()?;

    if output.status.success() {
        info!("✅ Successfully installed {} via snap", app);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("❌ Failed to install {} via snap: {}", app, error_msg);
        Err(anyhow::anyhow!("Snap installation failed: {}", error_msg))
    }
}

pub fn search_snap(query: &str) -> Result<Vec<String>> {
    info!("Searching snap for: {}", query);
    
    let output = Command::new("snap")
        .arg("find")
        .arg(query)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header line
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.first().map(|s| s.to_string())
            })
            .collect();
        
        Ok(packages)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to search snap: {}", error_msg);
        Ok(vec![])
    }
}

pub fn get_snap_info(app: &str) -> Result<String> {
    info!("Getting info for snap package: {}", app);
    
    let output = Command::new("snap")
        .arg("info")
        .arg(app)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Failed to get snap info: {}", error_msg))
    }
}

pub fn update_snap(app: &str) -> Result<()> {
    info!("Updating snap package: {}", app);
    
    let output = Command::new("snap")
        .arg("refresh")
        .arg(app)
        .output()?;

    if output.status.success() {
        info!("✅ Successfully updated {} via snap", app);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("❌ Failed to update {} via snap: {}", app, error_msg);
        Err(anyhow::anyhow!("Snap update failed: {}", error_msg))
    }
}

pub fn remove_snap(app: &str) -> Result<()> {
    info!("Removing snap package: {}", app);
    
    let output = Command::new("snap")
        .arg("remove")
        .arg(app)
        .output()?;

    if output.status.success() {
        info!("✅ Successfully removed {} via snap", app);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("❌ Failed to remove {} via snap: {}", app, error_msg);
        Err(anyhow::anyhow!("Snap removal failed: {}", error_msg))
    }
}