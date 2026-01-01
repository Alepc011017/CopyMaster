// src/core/autostart.rs
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use dirs;

#[derive(Debug, Clone)]
pub enum DesktopEnvironment {
    GNOME,
    KDE,
    XFCE,
    LXDE,
    LXQT,
    MATE,
    Cinnamon,
    Unknown,
}

impl DesktopEnvironment {
    pub fn detect() -> Self {
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            match de.to_lowercase().as_str() {
                "gnome" | "ubuntu:gnome" | "pop:gnome" => Self::GNOME,
                "kde" | "plasma" => Self::KDE,
                "xfce" => Self::XFCE,
                "lxde" => Self::LXDE,
                "lxqt" => Self::LXQT,
                "mate" => Self::MATE,
                "cinnamon" => Self::Cinnamon,
                _ => Self::Unknown,
            }
        } else if let Ok(de) = std::env::var("DESKTOP_SESSION") {
            match de.to_lowercase().as_str() {
                "gnome" | "ubuntu" => Self::GNOME,
                "plasma" => Self::KDE,
                "xfce" => Self::XFCE,
                "mate" => Self::MATE,
                _ => Self::Unknown,
            }
        } else {
            Self::Unknown
        }
    }
}

pub struct AutoStartManager {
    desktop_env: DesktopEnvironment,
    config_dir: PathBuf,
}

impl AutoStartManager {
    pub fn new() -> io::Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No se encontró el directorio de configuración"))?
            .join("autostart");
        
        Ok(Self {
            desktop_env: DesktopEnvironment::detect(),
            config_dir,
        })
    }
    
    /// Verificar si está configurado para auto-arranque
    pub fn is_enabled(&self) -> bool {
        let desktop_file = self.config_dir.join("copymaster.desktop");
        desktop_file.exists()
    }
    
    /// Habilitar auto-arranque
    pub fn enable(&self, minimized: bool) -> io::Result<()> {
        // Crear directorio si no existe
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)?;
        }
        
        // Crear archivo .desktop
        let desktop_file = self.config_dir.join("copymaster.desktop");
        let mut file = fs::File::create(&desktop_file)?;
        
        let exec_line = if minimized {
            "Exec=copymaster --minimized"
        } else {
            "Exec=copymaster"
        };
        
        let content = format!(
            "[Desktop Entry]\n\
            Type=Application\n\
            Name=CopyMaster\n\
            Comment=Administrador de copias avanzado\n\
            {}\n\
            Icon=copymaster\n\
            Terminal=false\n\
            StartupNotify=false\n\
            X-GNOME-Autostart-enabled=true\n\
            Hidden=false\n",
            exec_line
        );
        
        file.write_all(content.as_bytes())?;
        
        // Añadir permisos de ejecución
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&desktop_file)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&desktop_file, permissions)?;
        }
        
        println!("Auto-arranque habilitado en: {}", desktop_file.display());
        Ok(())
    }
    
    /// Deshabilitar auto-arranque
    pub fn disable(&self) -> io::Result<()> {
        let desktop_file = self.config_dir.join("copymaster.desktop");
        
        if desktop_file.exists() {
            fs::remove_file(&desktop_file)?;
            println!("Auto-arranque deshabilitado");
        }
        
        Ok(())
    }
    
    /// Configurar auto-arranque específico para el entorno de escritorio
    pub fn setup_for_desktop_env(&self) -> io::Result<()> {
        match self.desktop_env {
            DesktopEnvironment::GNOME => self.setup_gnome_autostart(),
            DesktopEnvironment::KDE => self.setup_kde_autostart(),
            DesktopEnvironment::XFCE => self.setup_xfce_autostart(),
            _ => self.setup_generic_autostart(),
        }
    }
    
    fn setup_gnome_autostart(&self) -> io::Result<()> {
        // GNOME usa el estándar XDG, ya lo tenemos cubierto
        self.enable(true)
    }
    
    fn setup_kde_autostart(&self) -> io::Result<()> {
        // KDE también usa XDG, pero podemos añadir configuración específica
        self.enable(true)
    }
    
    fn setup_xfce_autostart(&self) -> io::Result<()> {
        // XFCE tiene su propio gestor de sesiones
        let xfce_autostart_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("xfce4/autostart");
        
        if !xfce_autostart_dir.exists() {
            fs::create_dir_all(&xfce_autostart_dir)?;
        }
        
        let xfce_desktop = xfce_autostart_dir.join("copymaster.desktop");
        let content = "[Desktop Entry]\n\
                      Type=Application\n\
                      Name=CopyMaster\n\
                      Exec=copymaster --minimized\n\
                      StartupNotify=false\n";
        
        fs::write(xfce_desktop, content)?;
        
        Ok(())
    }
    
    fn setup_generic_autostart(&self) -> io::Result<()> {
        // Método genérico que funciona en varios entornos
        self.enable(true)
    }
    
    /// Crear entrada en el menú de aplicaciones
    pub fn create_app_menu_entry(&self) -> io::Result<()> {
        let applications_dir = dirs::data_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No se encontró el directorio de datos"))?
            .join("applications");
        
        if !applications_dir.exists() {
            fs::create_dir_all(&applications_dir)?;
        }
        
        let desktop_file = applications_dir.join("copymaster.desktop");
        
        let content = "[Desktop Entry]\n\
                      Type=Application\n\
                      Name=CopyMaster\n\
                      Comment=Administrador de copias avanzado\n\
                      Exec=copymaster\n\
                      Icon=copymaster\n\
                      Terminal=false\n\
                      Categories=Utility;FileTools;\n\
                      Keywords=copy;file;transfer;backup;\n\
                      StartupWMClass=copymaster\n";
        
        fs::write(desktop_file, content)?;
        
        Ok(())
    }
}