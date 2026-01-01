// src/core/device_detector.rs
use std::path::PathBuf;
use std::process::Command;
use nix::sys::statvfs::statvfs;

#[derive(Debug, Clone)]
pub enum DeviceType {
    Unknown,
    NVMeSSD,
    SataSSD,
    HDD,
    USB3,
    USB2,
    USB1,
    SDCard,
    RAMDisk,
    Optical,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: PathBuf,
    pub device_type: DeviceType,
    pub mount_point: Option<PathBuf>,
    pub filesystem: Option<String>,
    pub block_size: u64,
    pub total_space: u64,
    pub available_space: u64,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub estimated_speed_mbps: Option<f64>,
}

pub struct DeviceMonitor {
    known_devices: Vec<DeviceInfo>,
}

impl DeviceMonitor {
    pub fn new() -> Self {
        Self {
            known_devices: Vec::new(),
        }
    }

    pub async fn start_monitoring(&mut self, notification_channel: tokio::sync::mpsc::Sender<crate::core::daemon::DaemonNotification>) {
        // Implementar monitoreo de dispositivos (usando udev, dbus, etc.)
        // Por simplicidad, no implementamos el monitoreo completo aquí.
    }

    pub async fn detect_device(&mut self, path: &PathBuf) -> DeviceInfo {
        // Detección básica
        let mut info = DeviceInfo {
            path: path.clone(),
            device_type: DeviceType::Unknown,
            mount_point: self.get_mount_point(path).await,
            filesystem: self.get_filesystem_type(path).await,
            block_size: 4096,
            total_space: 0,
            available_space: 0,
            is_removable: false,
            is_read_only: false,
            estimated_speed_mbps: None,
        };

        if let Ok(stat) = statvfs(path) {
            info.block_size = stat.block_size() as u64;
            info.total_space = stat.blocks() * stat.block_size();
            info.available_space = stat.blocks_free() * stat.block_size();
        }

        #[cfg(target_os = "linux")]
        {
            info.device_type = self.detect_linux_device_type(path).await;
            info.is_removable = self.is_removable_linux(path).await;
            info.is_read_only = self.is_read_only_linux(path).await;
        }

        info.estimated_speed_mbps = self.estimate_speed(&info.device_type);

        info
    }

    async fn get_mount_point(&self, path: &PathBuf) -> Option<PathBuf> {
        // Implementar obtención del punto de montaje
        None
    }

    async fn get_filesystem_type(&self, path: &PathBuf) -> Option<String> {
        // Implementar obtención del tipo de sistema de archivos
        None
    }

    #[cfg(target_os = "linux")]
    async fn detect_linux_device_type(&self, path: &PathBuf) -> DeviceType {
        // Implementar detección para Linux
        DeviceType::Unknown
    }

    #[cfg(target_os = "linux")]
    async fn is_removable_linux(&self, path: &PathBuf) -> bool {
        false
    }

    #[cfg(target_os = "linux")]
    async fn is_read_only_linux(&self, path: &PathBuf) -> bool {
        false
    }

    fn estimate_speed(&self, device_type: &DeviceType) -> Option<f64> {
        match device_type {
            DeviceType::NVMeSSD => Some(3500.0),
            DeviceType::SataSSD => Some(550.0),
            DeviceType::HDD => Some(120.0),
            DeviceType::USB3 => Some(400.0),
            DeviceType::USB2 => Some(40.0),
            DeviceType::USB1 => Some(1.5),
            DeviceType::SDCard => Some(90.0),
            DeviceType::RAMDisk => Some(6000.0),
            DeviceType::Optical => Some(22.0),
            _ => None,
        }
    }
}