use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Linux(LinuxDistro),
    Windows,
    MacOS,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    CentOS,
    RHEL,
    Arch,
    OpenSUSE,
    Unknown,
}

pub trait PackageManager {
    fn install(&self, package: &str) -> Result<()>;
    fn remove(&self, package: &str) -> Result<()>;
    fn update(&self, package: Option<&str>) -> Result<()>;
    fn search(&self, query: &str) -> Result<Vec<String>>;
    fn list_installed(&self) -> Result<Vec<String>>;
    fn get_info(&self, package: &str) -> Result<String>;
    fn needs_privilege(&self) -> bool;
    fn get_name(&self) -> &'static str;
    fn get_priority(&self) -> u8;
}

pub fn detect_os() -> OperatingSystem {
    if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOS
    } else if cfg!(target_os = "linux") {
        OperatingSystem::Linux(detect_linux_distro())
    } else {
        OperatingSystem::Unknown
    }
}

pub fn detect_linux_distro() -> LinuxDistro {
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();
    
    if contents.contains("Ubuntu") {
        LinuxDistro::Ubuntu
    } else if contents.contains("Debian") {
        LinuxDistro::Debian
    } else if contents.contains("Fedora") {
        LinuxDistro::Fedora
    } else if contents.contains("CentOS") {
        LinuxDistro::CentOS
    } else if contents.contains("Red Hat Enterprise Linux") || contents.contains("RHEL") {
        LinuxDistro::RHEL
    } else if contents.contains("Arch") {
        LinuxDistro::Arch
    } else if contents.contains("openSUSE") || contents.contains("SUSE") {
        LinuxDistro::OpenSUSE
    } else {
        LinuxDistro::Unknown
    }
}

pub fn detect_distro() -> String {
    match detect_os() {
        OperatingSystem::Linux(distro) => match distro {
            LinuxDistro::Ubuntu | LinuxDistro::Debian => "apt".to_string(),
            LinuxDistro::Fedora | LinuxDistro::CentOS | LinuxDistro::RHEL => "dnf".to_string(),
            LinuxDistro::Arch => "pacman".to_string(),
            LinuxDistro::OpenSUSE => "zypper".to_string(),
            LinuxDistro::Unknown => "unknown".to_string(),
        },
        OperatingSystem::Windows => "winget".to_string(),
        OperatingSystem::MacOS => "brew".to_string(),
        OperatingSystem::Unknown => "unknown".to_string(),
    }
}

pub fn get_available_package_managers() -> Vec<&'static str> {
    let mut managers = Vec::new();
    
    match detect_os() {
        OperatingSystem::Linux(_) => {
            // Linux package managers
            if command_exists("apt") { managers.push("apt"); }
            if command_exists("dnf") { managers.push("dnf"); }
            if command_exists("pacman") { managers.push("pacman"); }
            if command_exists("zypper") { managers.push("zypper"); }
            if command_exists("snap") { managers.push("snap"); }
            if command_exists("flatpak") { managers.push("flatpak"); }
            // AppImage is always available on Linux
            managers.push("appimage");
        },
        OperatingSystem::Windows => {
            // Windows package managers
            if command_exists("winget") { managers.push("winget"); }
            if command_exists("choco") { managers.push("chocolatey"); }
            if command_exists("scoop") { managers.push("scoop"); }
        },
        OperatingSystem::MacOS => {
            // macOS package managers
            if command_exists("brew") { managers.push("brew"); }
            if command_exists("mas") { managers.push("mas"); }
        },
        OperatingSystem::Unknown => {},
    }
    
    managers
}

pub fn command_exists(cmd: &str) -> bool {
    if cfg!(target_os = "windows") {
        std::process::Command::new("where")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} > /dev/null 2>&1", cmd))
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub fn get_os_display_name() -> String {
    match detect_os() {
        OperatingSystem::Linux(distro) => {
            let distro_name = match distro {
                LinuxDistro::Ubuntu => "Ubuntu",
                LinuxDistro::Debian => "Debian",
                LinuxDistro::Fedora => "Fedora",
                LinuxDistro::CentOS => "CentOS",
                LinuxDistro::RHEL => "Red Hat Enterprise Linux",
                LinuxDistro::Arch => "Arch Linux",
                LinuxDistro::OpenSUSE => "openSUSE",
                LinuxDistro::Unknown => "Linux",
            };
            format!("{} Linux", distro_name)
        },
        OperatingSystem::Windows => {
            // Try to get Windows version
            if let Ok(output) = std::process::Command::new("ver").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                if !version.is_empty() {
                    return version.trim().to_string();
                }
            }
            "Windows".to_string()
        },
        OperatingSystem::MacOS => {
            // Try to get macOS version
            if let Ok(output) = std::process::Command::new("sw_vers")
                .args(&["-productName"])
                .output() 
            {
                let product = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(version_output) = std::process::Command::new("sw_vers")
                    .args(&["-productVersion"])
                    .output() 
                {
                    let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
                    return format!("{} {}", product, version);
                }
                return product;
            }
            "macOS".to_string()
        },
        OperatingSystem::Unknown => "Unknown OS".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_detection() {
        let os = detect_os();
        // Just ensure it doesn't panic and returns a valid OS
        match os {
            OperatingSystem::Linux(_) | 
            OperatingSystem::Windows | 
            OperatingSystem::MacOS | 
            OperatingSystem::Unknown => assert!(true),
        }
    }

    #[test]
    fn test_package_manager_detection() {
        let managers = get_available_package_managers();
        // Should return at least empty vector without panicking
        assert!(managers.len() >= 0);
    }
}