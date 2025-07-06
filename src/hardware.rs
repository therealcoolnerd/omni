use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub network: Vec<NetworkDevice>,
    pub storage: Vec<StorageDevice>,
    pub gpu: Vec<GpuDevice>,
    pub system: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub vendor: String,
    pub model: String,
    pub architecture: String,
    pub cores: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub vendor: String,
    pub model: String,
    pub driver: Option<String>,
    pub driver_needed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub vendor: String,
    pub model: String,
    pub driver: Option<String>,
    pub driver_needed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub vendor: String,
    pub model: String,
    pub driver: Option<String>,
    pub driver_needed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub vendor: String,
    pub model: String,
    pub bios_version: String,
}

pub struct HardwareDetector;

impl HardwareDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect all hardware and suggest appropriate drivers
    pub fn detect_hardware(&self) -> Result<HardwareInfo> {
        info!("Detecting server hardware configuration");

        let cpu = self.detect_cpu()?;
        let network = self.detect_network_devices()?;
        let storage = self.detect_storage_devices()?;
        let gpu = self.detect_gpu_devices()?;
        let system = self.detect_system_info()?;

        Ok(HardwareInfo {
            cpu,
            network,
            storage,
            gpu,
            system,
        })
    }

    /// Get recommended driver packages for current hardware
    pub fn get_recommended_drivers(&self, hardware: &HardwareInfo) -> Vec<String> {
        let mut drivers = Vec::new();

        // Network drivers
        for device in &hardware.network {
            if let Some(driver) = &device.driver_needed {
                drivers.push(driver.clone());
            }
        }

        // Storage drivers
        for device in &hardware.storage {
            if let Some(driver) = &device.driver_needed {
                drivers.push(driver.clone());
            }
        }

        // GPU drivers
        for device in &hardware.gpu {
            if let Some(driver) = &device.driver_needed {
                drivers.push(driver.clone());
            }
        }

        // Add common server drivers
        drivers.extend(self.get_common_server_drivers(&hardware.system));

        drivers
    }

    fn detect_cpu(&self) -> Result<CpuInfo> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

        let vendor = self
            .extract_cpu_field(&cpuinfo, "vendor_id")
            .unwrap_or("unknown".to_string());
        let model = self
            .extract_cpu_field(&cpuinfo, "model name")
            .unwrap_or("unknown".to_string());
        let architecture = std::env::consts::ARCH.to_string();

        // Count cores
        let cores = cpuinfo.matches("processor").count() as u32;

        Ok(CpuInfo {
            vendor,
            model,
            architecture,
            cores,
        })
    }

    fn detect_network_devices(&self) -> Result<Vec<NetworkDevice>> {
        let mut devices = Vec::new();

        // Use lspci to detect network devices
        if let Ok(output) = Command::new("lspci").arg("-nn").output() {
            let lspci_output = String::from_utf8_lossy(&output.stdout);

            for line in lspci_output.lines() {
                if line.contains("Network controller") || line.contains("Ethernet controller") {
                    if let Some(device) = self.parse_network_device(line) {
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }

    fn detect_storage_devices(&self) -> Result<Vec<StorageDevice>> {
        let mut devices = Vec::new();

        // Use lspci for storage controllers
        if let Ok(output) = Command::new("lspci").arg("-nn").output() {
            let lspci_output = String::from_utf8_lossy(&output.stdout);

            for line in lspci_output.lines() {
                if line.contains("RAID") || line.contains("SATA") || line.contains("NVMe") {
                    if let Some(device) = self.parse_storage_device(line) {
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }

    fn detect_gpu_devices(&self) -> Result<Vec<GpuDevice>> {
        let mut devices = Vec::new();

        // Use lspci for GPU devices
        if let Ok(output) = Command::new("lspci").arg("-nn").output() {
            let lspci_output = String::from_utf8_lossy(&output.stdout);

            for line in lspci_output.lines() {
                if line.contains("VGA")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    if let Some(device) = self.parse_gpu_device(line) {
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }

    fn detect_system_info(&self) -> Result<SystemInfo> {
        let vendor = self
            .read_dmi_info("sys_vendor")
            .unwrap_or("unknown".to_string());
        let model = self
            .read_dmi_info("product_name")
            .unwrap_or("unknown".to_string());
        let bios_version = self
            .read_dmi_info("bios_version")
            .unwrap_or("unknown".to_string());

        Ok(SystemInfo {
            vendor,
            model,
            bios_version,
        })
    }

    pub(crate) fn parse_network_device(&self, line: &str) -> Option<NetworkDevice> {
        // Parse vendor and model from lspci output
        // Example: "02:00.0 Ethernet controller [0200]: Intel Corporation 82574L Gigabit Network Connection [8086:10d3]"

        let bus = line.split_whitespace().next()?;
        let vendor_model = line.split("]:").nth(1)?.trim();
        let (vendor, model) = if let Some(corp_pos) = vendor_model.find("Corporation") {
            let vendor = vendor_model[..corp_pos + 11].trim();
            let model = vendor_model[corp_pos + 11..].trim();
            (vendor.to_string(), model.to_string())
        } else if let Some(space_pos) = vendor_model.find(' ') {
            let vendor = vendor_model[..space_pos].trim();
            let model = vendor_model[space_pos..].trim();
            (vendor.to_string(), model.to_string())
        } else {
            (vendor_model.to_string(), "unknown".to_string())
        };

        let driver_needed = self.suggest_network_driver(&vendor, &model);
        let driver = self.get_pci_driver(bus);

        Some(NetworkDevice {
            vendor,
            model,
            driver,
            driver_needed,
        })
    }

    pub(crate) fn parse_storage_device(&self, line: &str) -> Option<StorageDevice> {
        // Example line: "01:00.0 SATA controller [0106]: Intel Corporation XYZ [8086:1234]"
        let bus = line.split_whitespace().next()?;
        let vendor_model = line.split("]:").nth(1)?.trim();
        let (vendor, model) = if let Some(corp_pos) = vendor_model.find("Corporation") {
            let vendor = vendor_model[..corp_pos + 11].trim();
            let model = vendor_model[corp_pos + 11..].trim();
            (vendor.to_string(), model.to_string())
        } else if let Some(space_pos) = vendor_model.find(' ') {
            let vendor = vendor_model[..space_pos].trim();
            let model = vendor_model[space_pos..].trim();
            (vendor.to_string(), model.to_string())
        } else {
            (vendor_model.to_string(), "unknown".to_string())
        };

        let driver_needed = self.suggest_storage_driver(&vendor, &model);
        let driver = self.get_pci_driver(bus);

        Some(StorageDevice {
            vendor,
            model,
            driver,
            driver_needed,
        })
    }

    pub(crate) fn parse_gpu_device(&self, line: &str) -> Option<GpuDevice> {
        // Example line: "03:00.0 VGA compatible controller: NVIDIA Corporation GP102 [10de:1b06]"
        let bus = line.split_whitespace().next()?;
        let vendor_model = line.split("]:").nth(1)?.trim();

        let (vendor, model) = if let Some(pos) = vendor_model.find("Corporation") {
            let vendor = vendor_model[..pos + 11].trim();
            let model = vendor_model[pos + 11..].trim();
            (vendor.to_string(), model.to_string())
        } else if let Some(space_pos) = vendor_model.find(' ') {
            let vendor = vendor_model[..space_pos].trim();
            let model = vendor_model[space_pos..].trim();
            (vendor.to_string(), model.to_string())
        } else {
            (vendor_model.to_string(), "unknown".to_string())
        };

        let driver = self.get_pci_driver(bus);
        let driver_needed = if vendor.to_lowercase().contains("nvidia") {
            Some("nvidia-driver".to_string())
        } else if vendor.to_lowercase().contains("amd") || vendor.to_lowercase().contains("ati") {
            Some("amdgpu".to_string())
        } else {
            None
        };

        Some(GpuDevice {
            vendor,
            model,
            driver,
            driver_needed,
        })
    }

    fn suggest_network_driver(&self, vendor: &str, model: &str) -> Option<String> {
        match vendor.to_lowercase().as_str() {
            v if v.contains("intel") => Some("intel-ethernet".to_string()),
            v if v.contains("broadcom") => Some("broadcom-sta".to_string()),
            v if v.contains("realtek") => Some("r8168".to_string()),
            v if v.contains("mellanox") => Some("mlx5-core".to_string()),
            _ => None,
        }
    }

    fn suggest_storage_driver(&self, vendor: &str, _model: &str) -> Option<String> {
        match vendor.to_lowercase().as_str() {
            v if v.contains("intel") => Some("intel-storage".to_string()),
            v if v.contains("lsi") || v.contains("broadcom") => Some("megaraid".to_string()),
            v if v.contains("samsung") => Some("nvme".to_string()),
            v if v.contains("adaptec") => Some("aacraid".to_string()),
            _ => None,
        }
    }

    fn get_pci_driver(&self, bus: &str) -> Option<String> {
        let addr = if bus.starts_with("0000:") { bus.to_string() } else { format!("0000:{}", bus) };
        let path = format!("/sys/bus/pci/devices/{}/driver", addr);
        fs::read_link(&path).ok().and_then(|p| p.file_name().map(|f| f.to_string_lossy().into()))
    }

    fn get_common_server_drivers(&self, system: &SystemInfo) -> Vec<String> {
        let mut drivers = Vec::new();

        match system.vendor.to_lowercase().as_str() {
            v if v.contains("dell") => {
                drivers.push("dell-smbios".to_string());
                drivers.push("dcdbas".to_string());
            }
            v if v.contains("hp") || v.contains("hewlett") => {
                drivers.push("hpilo".to_string());
                drivers.push("hp-wmi".to_string());
            }
            v if v.contains("supermicro") => {
                drivers.push("ipmi_si".to_string());
                drivers.push("ipmi_devintf".to_string());
            }
            _ => {}
        }

        // Common server hardware support
        drivers.extend(vec![
            "firmware-misc-nonfree".to_string(), // General firmware
            "linux-firmware".to_string(),        // Kernel firmware
            "microcode".to_string(),             // CPU microcode
        ]);

        drivers
    }

    fn extract_cpu_field(&self, cpuinfo: &str, field: &str) -> Option<String> {
        for line in cpuinfo.lines() {
            if line.starts_with(field) {
                if let Some(value) = line.split(':').nth(1) {
                    return Some(value.trim().to_string());
                }
            }
        }
        None
    }

    fn read_dmi_info(&self, field: &str) -> Option<String> {
        let path = format!("/sys/devices/virtual/dmi/id/{}", field);
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    }
}

/// High-level function to detect hardware and suggest driver packages
pub fn detect_and_suggest_drivers() -> Result<Vec<String>> {
    let detector = HardwareDetector::new();
    let hardware = detector.detect_hardware()?;
    let drivers = detector.get_recommended_drivers(&hardware);

    info!("Detected {} recommended drivers", drivers.len());
    for driver in &drivers {
        info!("  - {}", driver);
    }

    Ok(drivers)
}
