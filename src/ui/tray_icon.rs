// src/ui/tray_icon.rs
use gtk4::{glib, prelude::*, Application, MenuButton, PopoverMenu, Image, Menu, MenuItem, Label};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SystemTray {
    pub status_icon: StatusIcon,
    pub menu: TrayMenu,
    pub is_visible: bool,
    pub icon_state: TrayIconState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayIconState {
    Idle,           // No hay transferencias
    Active,         // Transferencia en progreso
    Paused,         // Transferencia pausada
    Error,          // Error en transferencia
    Warning,        // Advertencia
}

impl SystemTray {
    pub fn new(app: &Application) -> Self {
        let status_icon = StatusIcon::new();
        let menu = TrayMenu::new(app);
        
        // Conectar eventos
        status_icon.connect_activate({
            let menu = menu.clone();
            move |_| {
                menu.toggle_visibility();
            }
        });
        
        Self {
            status_icon,
            menu,
            is_visible: true,
            icon_state: TrayIconState::Idle,
        }
    }
    
    pub fn show(&self) {
        self.status_icon.show();
        self.is_visible = true;
    }
    
    pub fn hide(&self) {
        self.status_icon.hide();
        self.is_visible = false;
    }
    
    pub fn set_tooltip(&self, text: &str) {
        self.status_icon.set_tooltip(text);
    }
    
    pub fn update_icon_based_on_activity(&mut self, has_active_transfers: bool, is_paused: bool, has_errors: bool) {
        let new_state = if has_errors {
            TrayIconState::Error
        } else if is_paused {
            TrayIconState::Paused
        } else if has_active_transfers {
            TrayIconState::Active
        } else {
            TrayIconState::Idle
        };
        
        if new_state != self.icon_state {
            self.icon_state = new_state;
            self.update_icon();
        }
    }
    
    fn update_icon(&self) {
        let icon_name = match self.icon_state {
            TrayIconState::Idle => "copymaster-idle",
            TrayIconState::Active => "copymaster-active",
            TrayIconState::Paused => "copymaster-paused",
            TrayIconState::Error => "copymaster-error",
            TrayIconState::Warning => "copymaster-warning",
        };
        
        // Intentar cargar icono SVG primero, luego PNG
        self.status_icon.set_icon_name(icon_name);
        
        // Actualizar tooltip según estado
        let tooltip = match self.icon_state {
            TrayIconState::Idle => "CopyMaster - Inactivo",
            TrayIconState::Active => "CopyMaster - Copiando archivos",
            TrayIconState::Paused => "CopyMaster - Pausado",
            TrayIconState::Error => "CopyMaster - Error en transferencia",
            TrayIconState::Warning => "CopyMaster - Advertencia",
        };
        
        self.set_tooltip(tooltip);
    }
    
    pub fn show_notification(&self, title: &str, message: &str, icon_name: Option<&str>) {
        // Usar libnotify para notificaciones del sistema
        #[cfg(feature = "notifications")]
        {
            if let Err(e) = notify_rust::Notification::new()
                .summary(title)
                .body(message)
                .icon(icon_name.unwrap_or("copymaster"))
                .show() 
            {
                eprintln!("Error al mostrar notificación: {}", e);
            }
        }
        
        // Fallback: solo imprimir en consola
        #[cfg(not(feature = "notifications"))]
        {
            println!("{}: {}", title, message);
        }
    }
}

// Implementación GTK nativa para StatusIcon
pub struct StatusIcon {
    icon: gtk::StatusIcon,
    menu: gtk::Menu,
}

impl StatusIcon {
    pub fn new() -> Self {
        // Intentar usar StatusIcon si está disponible
        let icon = gtk::StatusIcon::new();
        
        // Intentar diferentes nombres de icono hasta encontrar uno que funcione
        let icon_names = vec![
            "copymaster",
            "copymaster-symbolic",
            "system-file-manager",
            "folder",
            "document-save",
        ];
        
        for icon_name in icon_names {
            icon.set_from_icon_name(Some(icon_name));
            if icon.get_pixbuf().is_some() || icon.get_icon_name().is_some() {
                break;
            }
        }
        
        icon.set_tooltip_text(Some("CopyMaster - Administrador de copias"));
        icon.set_visible(true);
        
        // Crear menú contextual
        let menu = gtk::Menu::new();
        
        Self { icon, menu }
    }
    
    pub fn set_icon_name(&self, icon_name: &str) {
        // Intentar diferentes variaciones del nombre del icono
        let icon_variations = vec![
            icon_name.to_string(),
            format!("{}-symbolic", icon_name),
            icon_name.replace("-symbolic", ""),
        ];
        
        for variation in icon_variations {
            self.icon.set_from_icon_name(Some(&variation));
            if self.icon.get_pixbuf().is_some() || self.icon.get_icon_name().is_some() {
                break;
            }
        }
    }
    
    pub fn set_tooltip(&self, text: &str) {
        self.icon.set_tooltip_text(Some(text));
    }
    
    pub fn show(&self) {
        self.icon.set_visible(true);
    }
    
    pub fn hide(&self) {
        self.icon.set_visible(false);
    }
    
    pub fn connect_activate<F: Fn() + 'static>(&self, f: F) {
        self.icon.connect_activate(move |_| {
            f();
        });
    }
}

// Menú para la bandeja del sistema
pub struct TrayMenu {
    menu: gtk::PopoverMenu,
    app: Application,
}

impl TrayMenu {
    pub fn new(app: &Application) -> Self {
        let menu = gtk::PopoverMenu::new();
        
        // Crear modelo de menú
        let menu_model = gio::Menu::new();
        
        // Sección principal
        menu_model.append(Some("Mostrar ventana"), Some("app.show"));
        menu_model.append(Some("Ocultar ventana"), Some("app.hide"));
        
        // Sección de transferencias
        let transfers_section = gio::Menu::new();
        transfers_section.append(Some("Pausar todas"), Some("app.pause_all"));
        transfers_section.append(Some("Reanudar todas"), Some("app.resume_all"));
        transfers_section.append(Some("Cancelar todas"), Some("app.cancel_all"));
        menu_model.append_section(Some("Transferencias"), &transfers_section);
        
        // Sección de dispositivo
        let device_section = gio::Menu::new();
        device_section.append(Some("Abrir dispositivo..."), Some("app.open_device"));
        device_section.append(Some("Expulsar dispositivo"), Some("app.eject_device"));
        menu_model.append_section(Some("Dispositivo"), &device_section);
        
        // Sección de configuración
        let settings_section = gio::Menu::new();
        settings_section.append(Some("Configuración"), Some("app.settings"));
        settings_section.append(Some("Auto-arranque"), Some("app.autostart"));
        menu_model.append_section(Some("Configuración"), &settings_section);
        
        // Separador y salir
        menu_model.append(Some("Salir"), Some("app.quit"));
        
        menu.set_menu_model(Some(&menu_model));
        
        Self {
            menu,
            app: app.clone(),
        }
    }
    
    pub fn toggle_visibility(&self) {
        if self.menu.is_visible() {
            self.menu.popdown();
        } else {
            self.menu.popup();
        }
    }
    
    pub fn connect_to_menu_button(&self, button: &MenuButton) {
        button.set_popover(Some(&self.menu));
    }
}