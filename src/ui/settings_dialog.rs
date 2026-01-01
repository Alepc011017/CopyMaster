use gtk4::{prelude::*, Dialog, Label, Button, Box, ComboBoxText, CheckButton, Entry, Frame, Grid};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SettingsDialog {
    dialog: Dialog,
    config_manager: Arc<Mutex<crate::core::config::ConfigManager>>,
    conflict_combo: ComboBoxText,
    confirm_check: CheckButton,
    rename_entry: Entry,
    autostart_check: CheckButton,
    minimized_check: CheckButton,
    minimize_tray_check: CheckButton,
    notifications_check: CheckButton,
}

impl SettingsDialog {
    pub fn new(
        parent: Option<&gtk::Window>,
        config_manager: Arc<Mutex<crate::core::config::ConfigManager>>,
    ) -> Self {
        let dialog = Dialog::new();
        
        if let Some(p) = parent {
            dialog.set_transient_for(Some(p));
        }
        
        dialog.set_title(Some("Configuración de CopyMaster"));
        dialog.set_default_size(500, 600);
        
        let content_area = dialog.content_area();
        content_area.set_spacing(20);
        content_area.set_margin_all(20);
        
        // Título
        let title = Label::new(Some("Configuración"));
        title.add_css_class("title-1");
        
        // Sección: Conflictos de archivos
        let conflict_section = Self::create_conflict_section();
        
        // Sección: Auto-arranque
        let autostart_section = Self::create_autostart_section();
        
        // Sección: Interfaz
        let ui_section = Self::create_ui_section();
        
        // Botones
        let button_box = Box::new(gtk::Orientation::Horizontal, 10);
        
        let save_btn = Button::with_label("Guardar");
        save_btn.add_css_class("suggested-action");
        
        let cancel_btn = Button::with_label("Cancelar");
        
        button_box.append(&save_btn);
        button_box.append(&cancel_btn);
        
        // Organizar
        content_area.append(&title);
        content_area.append(&conflict_section);
        content_area.append(&autostart_section);
        content_area.append(&ui_section);
        content_area.append(&button_box);
        
        // Crear instancia
        let mut instance = Self {
            dialog,
            config_manager,
            conflict_combo: ComboBoxText::new(),
            confirm_check: CheckButton::with_label("Siempre pedir confirmación"),
            rename_entry: Entry::new(),
            autostart_check: CheckButton::with_label("Iniciar CopyMaster al encender la computadora"),
            minimized_check: CheckButton::with_label("Iniciar minimizado en la bandeja del sistema"),
            minimize_tray_check: CheckButton::with_label("Minimizar a la bandeja en lugar de cerrar"),
            notifications_check: CheckButton::with_label("Mostrar notificaciones del sistema"),
        };
        
        // Cargar configuración actual
        instance.load_current_config().await;
        
        // Conectar señales
        save_btn.connect_clicked({
            let instance = instance.clone();
            move |_| {
                let instance = instance.clone();
                tokio::spawn(async move {
                    instance.save_config().await;
                });
            }
        });
        
        cancel_btn.connect_clicked({
            let dialog = instance.dialog.clone();
            move |_| {
                dialog.emit_response(gtk::ResponseType::Cancel);
            }
        });
        
        instance
    }
    
    fn create_conflict_section() -> gtk::Frame {
        let frame = Frame::new(Some("Conflicto de Archivos"));
        frame.add_css_class("settings-frame");
        
        let content = Box::new(gtk::Orientation::Vertical, 10);
        content.set_margin_all(10);
        
        // Acción por defecto
        let action_label = Label::new(Some("Acción por defecto cuando existe un archivo:"));
        action_label.set_xalign(0.0);
        
        let conflict_combo = ComboBoxText::new();
        conflict_combo.append(Some("ask"), "Preguntar siempre");
        conflict_combo.append(Some("overwrite"), "Sobrescribir siempre");
        conflict_combo.append(Some("skip"), "Saltar siempre");
        conflict_combo.append(Some("rename_new"), "Renombrar siempre el nuevo");
        conflict_combo.append(Some("rename_old"), "Renombrar siempre el antiguo");
        
        // Checkbox para confirmación
        let confirm_check = CheckButton::with_label("Siempre pedir confirmación");
        
        // Patrón de renombrado
        let rename_label = Label::new(Some("Patrón para renombrar:"));
        rename_label.set_xalign(0.0);
        
        let rename_entry = Entry::new();
        rename_entry.set_tooltip_text(Some("Use {name} para el nombre original y {counter} para el número"));
        
        content.append(&action_label);
        content.append(&conflict_combo);
        content.append(&confirm_check);
        content.append(&rename_label);
        content.append(&rename_entry);
        
        frame.set_child(Some(&content));
        frame
    }
    
    fn create_autostart_section() -> gtk::Frame {
        let frame = Frame::new(Some("Auto-arranque"));
        frame.add_css_class("settings-frame");
        
        let content = Box::new(gtk::Orientation::Vertical, 10);
        content.set_margin_all(10);
        
        let autostart_check = CheckButton::with_label("Iniciar CopyMaster al encender la computadora");
        let minimized_check = CheckButton::with_label("Iniciar minimizado en la bandeja del sistema");
        
        content.append(&autostart_check);
        content.append(&minimized_check);
        
        frame.set_child(Some(&content));
        frame
    }
    
    fn create_ui_section() -> gtk::Frame {
        let frame = Frame::new(Some("Interfaz de Usuario"));
        frame.add_css_class("settings-frame");
        
        let content = Box::new(gtk::Orientation::Vertical, 10);
        content.set_margin_all(10);
        
        let minimize_tray_check = CheckButton::with_label("Minimizar a la bandeja en lugar de cerrar");
        let notifications_check = CheckButton::with_label("Mostrar notificaciones del sistema");
        
        content.append(&minimize_tray_check);
        content.append(&notifications_check);
        
        frame.set_child(Some(&content));
        frame
    }
    
    async fn load_current_config(&mut self) {
        if let Ok(config) = self.config_manager.lock().await {
            let config = config.get_config();
            
            // Configuración de conflictos
            match config.conflict_resolution.default_action {
                crate::core::config::ConflictAction::Ask => 
                    self.conflict_combo.set_active_id(Some("ask")),
                crate::core::config::ConflictAction::Overwrite => 
                    self.conflict_combo.set_active_id(Some("overwrite")),
                crate::core::config::ConflictAction::Skip => 
                    self.conflict_combo.set_active_id(Some("skip")),
                crate::core::config::ConflictAction::RenameNew => 
                    self.conflict_combo.set_active_id(Some("rename_new")),
                crate::core::config::ConflictAction::RenameOld => 
                    self.conflict_combo.set_active_id(Some("rename_old")),
                _ => self.conflict_combo.set_active_id(Some("ask")),
            }
            
            self.confirm_check.set_active(config.conflict_resolution.ask_for_confirmation);
            self.rename_entry.set_text(&config.conflict_resolution.rename_pattern);
            
            // Auto-arranque
            self.autostart_check.set_active(config.autostart_enabled);
            self.minimized_check.set_active(config.start_minimized);
            
            // Interfaz
            self.minimize_tray_check.set_active(config.minimize_to_tray);
            self.notifications_check.set_active(config.show_notifications);
        }
    }
    
    async fn save_config(&self) {
        if let Ok(mut config) = self.config_manager.lock().await {
            // Conflictos
            let action = match self.conflict_combo.active_id().as_deref() {
                Some("overwrite") => crate::core::config::ConflictAction::Overwrite,
                Some("skip") => crate::core::config::ConflictAction::Skip,
                Some("rename_new") => crate::core::config::ConflictAction::RenameNew,
                Some("rename_old") => crate::core::config::ConflictAction::RenameOld,
                _ => crate::core::config::ConflictAction::Ask,
            };
            
            config.update_conflict_resolution(
                action,
                self.confirm_check.is_active()
            ).ok();
            
            // Actualizar otros campos
            config.get_config_mut().conflict_resolution.rename_pattern = 
                self.rename_entry.text().to_string();
            
            config.get_config_mut().autostart_enabled = self.autostart_check.is_active();
            config.get_config_mut().start_minimized = self.minimized_check.is_active();
            config.get_config_mut().minimize_to_tray = self.minimize_tray_check.is_active();
            config.get_config_mut().show_notifications = self.notifications_check.is_active();
            
            // Guardar configuración
            if let Err(e) = config.save_config() {
                eprintln!("Error al guardar configuración: {}", e);
            } else {
                self.dialog.emit_response(gtk::ResponseType::Accept);
            }
        }
    }
    
    pub async fn run(&self) -> Option<()> {
        let response = self.dialog.run_future().await;
        
        match response {
            gtk::ResponseType::Accept => Some(()),
            _ => None,
        }
    }
}

impl Clone for SettingsDialog {
    fn clone(&self) -> Self {
        Self {
            dialog: self.dialog.clone(),
            config_manager: self.config_manager.clone(),
            conflict_combo: self.conflict_combo.clone(),
            confirm_check: self.confirm_check.clone(),
            rename_entry: self.rename_entry.clone(),
            autostart_check: self.autostart_check.clone(),
            minimized_check: self.minimized_check.clone(),
            minimize_tray_check: self.minimize_tray_check.clone(),
            notifications_check: self.notifications_check.clone(),
        }
    }
}