use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub autostart_enabled: bool,
    pub start_minimized: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
    pub default_copy_options: crate::core::copy_engine::CopyOptions,
    pub remembered_devices: Vec<DevicePreference>,
    pub window_state: WindowState,
    // NUEVO: Configuración de conflictos global
    pub conflict_resolution: GlobalConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePreference {
    pub device_id: String,
    pub default_action: crate::core::drag_drop::DropAction,
    pub remember_choice: bool,
}

// NUEVA ESTRUCTURA: Configuración de resolución de conflictos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConflictResolution {
    pub default_action: ConflictAction,
    pub ask_for_confirmation: bool,
    pub rename_pattern: String, // Ejemplo: "{name} ({counter})"
}

impl Default for GlobalConflictResolution {
    fn default() -> Self {
        Self {
            default_action: ConflictAction::Ask,
            ask_for_confirmation: true,
            rename_pattern: "{name} ({counter})".to_string(),
        }
    }
}

// NUEVO ENUM: Acciones para conflictos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictAction {
    Ask,           // Preguntar siempre
    Overwrite,     // Sobrescribir
    OverwriteAll,  // Sobrescribir siempre (esta copia)
    Skip,          // Saltar
    SkipAll,       // Saltar siempre (esta copia)
    RenameNew,     // Renombrar el nuevo
    RenameOld,     // Renombrar el antiguo
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart_enabled: false,
            start_minimized: false,
            minimize_to_tray: true,
            show_notifications: true,
            default_copy_options: crate::core::copy_engine::CopyOptions::default(),
            remembered_devices: Vec::new(),
            window_state: WindowState {
                width: 1000,
                height: 700,
                x: -1, // Centrado
                y: -1, // Centrado
                maximized: false,
            },
            conflict_resolution: GlobalConflictResolution::default(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("No se pudo encontrar directorio de configuración")?
            .join("copymaster");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        let config_file = config_dir.join("config.json");
        
        let config = if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            serde_json::from_str(&content)?
        } else {
            let default_config = AppConfig::default();
            // Guardar configuración por defecto
            let content = serde_json::to_string_pretty(&default_config)?;
            fs::write(&config_file, content)?;
            default_config
        };
        
        Ok(Self {
            config_path: config_file,
            config,
        })
    }
    
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }
    
    pub fn save_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    pub fn update_conflict_resolution(
        &mut self, 
        default_action: ConflictAction,
        ask_for_confirmation: bool
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.conflict_resolution.default_action = default_action;
        self.config.conflict_resolution.ask_for_confirmation = ask_for_confirmation;
        self.save_config()
    }
    
    pub fn update_autostart(&mut self, enabled: bool, start_minimized: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.config.autostart_enabled = enabled;
        self.config.start_minimized = start_minimized;
        self.save_config()?;
        
        // Aplicar cambios en el sistema
        let autostart = crate::core::autostart::AutoStartManager::new()?;
        
        if enabled {
            autostart.enable(start_minimized)?;
        } else {
            autostart.disable()?;
        }
        
        Ok(())
    }
    
    pub fn get_device_preference(&self, device_id: &str) -> Option<&DevicePreference> {
        self.config.remembered_devices
            .iter()
            .find(|d| d.device_id == device_id)
    }
    
    pub fn set_device_preference(&mut self, preference: DevicePreference) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.config.remembered_devices
            .iter()
            .position(|d| d.device_id == preference.device_id) 
        {
            self.config.remembered_devices[index] = preference;
        } else {
            self.config.remembered_devices.push(preference);
        }
        
        self.save_config()
    }
}