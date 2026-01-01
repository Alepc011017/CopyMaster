// src/ui/devices_panel.rs
use gtk4::{prelude::*, Box, Label, Image, Button};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct DevicesPanel {
    pub container: gtk::Box,
    pub devices: Arc<Mutex<HashMap<String, crate::core::device_detector::DeviceInfo>>>,
}

impl DevicesPanel {
    pub fn new() -> Self {
        let container = Box::new(gtk::Orientation::Vertical, 5);
        container.set_vexpand(true);
        
        Self {
            container,
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn add_device(&self, device_info: crate::core::device_detector::DeviceInfo) {
        let device_widget = self.create_device_widget(&device_info).await;
        self.container.append(&device_widget);
        
        let mut devices = self.devices.lock().await;
        devices.insert(device_info.path.to_string_lossy().to_string(), device_info);
    }
    
    async fn create_device_widget(&self, device: &crate::core::device_detector::DeviceInfo) -> gtk::Box {
        let device_box = Box::new(gtk::Orientation::Horizontal, 10);
        device_box.set_margin_all(10);
        device_box.add_css_class("device-box");
        
        // Icono del dispositivo
        let icon_name = match device.device_type {
            crate::core::device_detector::DeviceType::NVMeSSD => "drive-harddisk-ieee1394",
            crate::core::device_detector::DeviceType::SataSSD => "drive-harddisk-solidstate",
            crate::core::device_detector::DeviceType::HDD => "drive-harddisk",
            crate::core::device_detector::DeviceType::USB3 => "drive-removable-media-usb",
            crate::core::device_detector::DeviceType::USB2 => "drive-removable-media-usb",
            crate::core::device_detector::DeviceType::USB1 => "drive-removable-media",
            crate::core::device_detector::DeviceType::SDCard => "media-flash-sd-mmc",
            crate::core::device_detector::DeviceType::RAMDisk => "drive-harddisk-system",
            crate::core::device_detector::DeviceType::Optical => "media-optical",
            crate::core::device_detector::DeviceType::Unknown => "drive-harddisk",
        };
        
        let icon = Image::from_icon_name(icon_name);
        icon.add_css_class("device-icon");
        
        // Información del dispositivo
        let info_box = Box::new(gtk::Orientation::Vertical, 5);
        
        let name_label = Label::new(Some(&device.path.to_string_lossy()));
        name_label.set_xalign(0.0);
        name_label.add_css_class("device-name");
        
        let info_text = format!(
            "Tipo: {:?} | Espacio: {} GB libres de {} GB",
            device.device_type,
            device.available_space / 1_000_000_000,
            device.total_space / 1_000_000_000
        );
        
        let info_label = Label::new(Some(&info_text));
        info_label.set_xalign(0.0);
        info_label.add_css_class("device-info");
        
        info_box.append(&name_label);
        info_box.append(&info_label);
        
        // Botones de acción
        let action_box = Box::new(gtk::Orientation::Horizontal, 5);
        
        let open_btn = Button::with_label("Abrir");
        open_btn.add_css_class("action-button");
        
        let eject_btn = Button::with_label("Expulsar");
        eject_btn.add_css_class("action-button");
        
        action_box.append(&open_btn);
        action_box.append(&eject_btn);
        
        device_box.append(&icon);
        device_box.append(&info_box);
        device_box.append(&action_box);
        
        device_box
    }
}