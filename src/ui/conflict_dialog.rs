use gtk4::{prelude::*, Dialog, Label, Button, Box, CheckButton, Image, ResponseType, Frame};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct ConflictDialogRequest {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub transfer_name: String,
    pub response_sender: oneshot::Sender<crate::core::device_queue::ConflictResolutionResult>,
}

#[derive(Debug, Clone)]
pub struct ConflictDialogResponse {
    pub action: crate::core::config::ConflictAction,
    pub remember_for_transfer: bool,
    pub remember_globally: bool,
}

pub struct ConflictDialog {
    dialog: Dialog,
    response: Option<ConflictDialogResponse>,
    remember_transfer_check: CheckButton,
    remember_global_check: CheckButton,
}

impl ConflictDialog {
    pub fn new(
        parent: Option<&gtk::Window>,
        source: &PathBuf,
        dest: &PathBuf,
        transfer_name: &str,
    ) -> Self {
        let dialog = Dialog::new();
        
        if let Some(p) = parent {
            dialog.set_transient_for(Some(p));
        }
        
        dialog.set_title(Some(&format!("Conflicto en '{}'", transfer_name)));
        dialog.set_default_size(600, 400);
        dialog.set_modal(true);
        
        let content_area = dialog.content_area();
        content_area.set_spacing(15);
        content_area.set_margin_all(20);
        
        // Encabezado
        let header_box = Box::new(gtk::Orientation::Horizontal, 10);
        let warning_icon = Image::from_icon_name("dialog-warning");
        warning_icon.set_pixel_size(48);
        
        let title_label = Label::new(Some("¡Archivo ya existe!"));
        title_label.add_css_class("title-4");
        
        header_box.append(&warning_icon);
        header_box.append(&title_label);
        
        // Mensaje
        let message = format!(
            "El archivo ya existe en la ubicación de destino:\n\n{}\n\n¿Qué quieres hacer?",
            dest.display()
        );
        
        let message_label = Label::new(Some(&message));
        message_label.set_wrap(true);
        message_label.set_xalign(0.0);
        
        // Información detallada
        let info_box = Self::create_file_info_box(source, dest);
        
        // Opciones de acción
        let actions_box = Self::create_actions_box(&dialog);
        
        // Opciones de recordar
        let (remember_box, remember_transfer_check, remember_global_check) = Self::create_remember_box();
        
        // Organizar todo
        content_area.append(&header_box);
        content_area.append(&message_label);
        content_area.append(&info_box);
        content_area.append(&actions_box);
        content_area.append(&remember_box);
        
        Self {
            dialog,
            response: None,
            remember_transfer_check,
            remember_global_check,
        }
    }
    
    fn create_file_info_box(source: &PathBuf, dest: &PathBuf) -> gtk::Frame {
        let frame = Frame::new(None);
        frame.add_css_class("conflict-file-info");
        
        let info_box = Box::new(gtk::Orientation::Vertical, 10);
        info_box.set_margin_all(10);
        
        // Información del archivo fuente
        if let Ok(metadata) = std::fs::metadata(source) {
            let source_info = Self::create_file_info_item("Origen:", source, &metadata);
            info_box.append(&source_info);
        }
        
        // Información del archivo destino
        if let Ok(metadata) = std::fs::metadata(dest) {
            let dest_info = Self::create_file_info_item("Destino:", dest, &metadata);
            info_box.append(&dest_info);
        }
        
        frame.set_child(Some(&info_box));
        frame
    }
    
    fn create_file_info_item(label: &str, path: &PathBuf, metadata: &std::fs::Metadata) -> gtk::Box {
        let item_box = Box::new(gtk::Orientation::Horizontal, 10);
        
        let label_widget = Label::new(Some(label));
        label_widget.set_xalign(0.0);
        label_widget.set_width_chars(15);
        
        let filename = path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let size = metadata.len();
        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.elapsed().ok())
            .map(|d| format!("hace {} minutos", d.as_secs() / 60))
            .unwrap_or_else(|| "desconocido".to_string());
        
        let info_text = format!("{} ({} bytes, modificado: {})", 
            filename, size, modified);
        
        let info_label = Label::new(Some(&info_text));
        info_label.set_xalign(0.0);
        info_label.set_wrap(true);
        
        item_box.append(&label_widget);
        item_box.append(&info_label);
        
        item_box
    }
    
    fn create_actions_box(dialog: &Dialog) -> gtk::Box {
        let actions_box = Box::new(gtk::Orientation::Vertical, 8);
        actions_box.add_css_class("conflict-actions");
        
        // Botón: Sobrescribir
        let overwrite_btn = Button::with_label("Sobrescribir");
        overwrite_btn.set_tooltip_text(Some("Reemplazar el archivo existente"));
        overwrite_btn.add_css_class("conflict-action-button");
        overwrite_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(0)); // Overwrite
            }
        });
        
        // Botón: Sobrescribir siempre (esta copia)
        let overwrite_all_btn = Button::with_label("Sobrescribir siempre (esta copia)");
        overwrite_all_btn.set_tooltip_text(Some("Sobrescribir todos los conflictos en esta copia"));
        overwrite_all_btn.add_css_class("conflict-action-button");
        overwrite_all_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(1)); // OverwriteAll
            }
        });
        
        // Botón: Saltar
        let skip_btn = Button::with_label("Saltar");
        skip_btn.set_tooltip_text(Some("No copiar este archivo"));
        skip_btn.add_css_class("conflict-action-button");
        skip_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(2)); // Skip
            }
        });
        
        // Botón: Saltar siempre (esta copia)
        let skip_all_btn = Button::with_label("Saltar siempre (esta copia)");
        skip_all_btn.set_tooltip_text(Some("Saltar todos los conflictos en esta copia"));
        skip_all_btn.add_css_class("conflict-action-button");
        skip_all_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(3)); // SkipAll
            }
        });
        
        // Botón: Renombrar nuevo
        let rename_new_btn = Button::with_label("Renombrar archivo nuevo");
        rename_new_btn.set_tooltip_text(Some("Renombrar el archivo que se está copiando"));
        rename_new_btn.add_css_class("conflict-action-button");
        rename_new_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(4)); // RenameNew
            }
        });
        
        // Botón: Renombrar antiguo
        let rename_old_btn = Button::with_label("Renombrar archivo existente");
        rename_old_btn.set_tooltip_text(Some("Renombrar el archivo que ya existe"));
        rename_old_btn.add_css_class("conflict-action-button");
        rename_old_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Other(5)); // RenameOld
            }
        });
        
        // Botón: Cancelar
        let cancel_btn = Button::with_label("Cancelar copia");
        cancel_btn.add_css_class("destructive-action");
        cancel_btn.add_css_class("conflict-action-button");
        cancel_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(ResponseType::Cancel);
            }
        });
        
        actions_box.append(&overwrite_btn);
        actions_box.append(&overwrite_all_btn);
        actions_box.append(&skip_btn);
        actions_box.append(&skip_all_btn);
        actions_box.append(&rename_new_btn);
        actions_box.append(&rename_old_btn);
        actions_box.append(&cancel_btn);
        
        actions_box
    }
    
    fn create_remember_box() -> (gtk::Frame, CheckButton, CheckButton) {
        let frame = Frame::new(None);
        frame.add_css_class("conflict-remember-section");
        
        let remember_box = Box::new(gtk::Orientation::Vertical, 8);
        remember_box.set_margin_all(10);
        
        let remember_label = Label::new(Some("Recordar esta elección:"));
        remember_label.set_xalign(0.0);
        
        let remember_transfer_check = CheckButton::with_label("Para esta copia");
        let remember_global_check = CheckButton::with_label("Para todas las copias (configuración global)");
        
        remember_box.append(&remember_label);
        remember_box.append(&remember_transfer_check);
        remember_box.append(&remember_global_check);
        
        frame.set_child(Some(&remember_box));
        
        (frame, remember_transfer_check, remember_global_check)
    }
    
    pub async fn run(&mut self) -> Option<ConflictDialogResponse> {
        let response = self.dialog.run_future().await;
        
        let response = match response {
            ResponseType::Other(0) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::Overwrite,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            ResponseType::Other(1) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::OverwriteAll,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            ResponseType::Other(2) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::Skip,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            ResponseType::Other(3) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::SkipAll,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            ResponseType::Other(4) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::RenameNew,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            ResponseType::Other(5) => Some(ConflictDialogResponse {
                action: crate::core::config::ConflictAction::RenameOld,
                remember_for_transfer: self.remember_transfer_check.is_active(),
                remember_globally: self.remember_global_check.is_active(),
            }),
            _ => None,
        };
        
        self.response = response.clone();
        response
    }
}

// Servicio para manejar diálogos de conflicto
pub struct ConflictDialogService {
    request_receiver: tokio::sync::mpsc::Receiver<ConflictDialogRequest>,
    window: Option<gtk::Window>,
    config_manager: Arc<tokio::sync::Mutex<crate::core::config::ConfigManager>>,
}

impl ConflictDialogService {
    pub fn new(
        window: Option<gtk::Window>,
        config_manager: Arc<tokio::sync::Mutex<crate::core::config::ConfigManager>>,
    ) -> (Self, tokio::sync::mpsc::Sender<ConflictDialogRequest>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        
        let service = Self {
            request_receiver: receiver,
            window,
            config_manager,
        };
        
        (service, sender)
    }
    
    pub async fn run(mut self) {
        while let Some(request) = self.request_receiver.recv().await {
            self.handle_request(request).await;
        }
    }
    
    async fn handle_request(&self, request: ConflictDialogRequest) {
        // Crear diálogo
        let mut dialog = ConflictDialog::new(
            self.window.as_ref(),
            &request.source,
            &request.destination,
            &request.transfer_name,
        );
        
        // Ejecutar diálogo
        if let Some(response) = dialog.run().await {
            // Procesar respuesta
            let result = match response.action {
                crate::core::config::ConflictAction::Overwrite => 
                    crate::core::device_queue::ConflictResolutionResult::Overwrite,
                crate::core::config::ConflictAction::OverwriteAll => 
                    crate::core::device_queue::ConflictResolutionResult::Overwrite,
                crate::core::config::ConflictAction::Skip => 
                    crate::core::device_queue::ConflictResolutionResult::Skip,
                crate::core::config::ConflictAction::SkipAll => 
                    crate::core::device_queue::ConflictResolutionResult::Skip,
                crate::core::config::ConflictAction::RenameNew => 
                    crate::core::device_queue::ConflictResolutionResult::RenameNew,
                crate::core::config::ConflictAction::RenameOld => 
                    crate::core::device_queue::ConflictResolutionResult::RenameOld,
                _ => crate::core::device_queue::ConflictResolutionResult::Cancelled,
            };
            
            // Si el usuario quiere recordar globalmente, actualizar configuración
            if response.remember_globally {
                if let Ok(mut config) = self.config_manager.lock().await {
                    config.update_conflict_resolution(
                        response.action.clone(),
                        false // No preguntar más
                    ).ok();
                }
            }
            
            // Enviar respuesta
            let _ = request.response_sender.send(result);
        } else {
            // Diálogo cancelado
            let _ = request.response_sender.send(
                crate::core::device_queue::ConflictResolutionResult::Cancelled
            );
        }
    }
}