// src/ui/drop_dialog.rs
use gtk4::{prelude::*, Dialog, Label, Button, Box, Image, CheckButton};
use std::path::PathBuf;

pub struct DropDialog {
    dialog: Dialog,
    response: DropDialogResponse,
}

#[derive(Debug, Clone)]
pub struct DropDialogResponse {
    pub action: DropAction,
    pub remember_choice: bool,
    pub options: crate::core::drag_drop::TransferOptions,
}

#[derive(Debug, Clone)]
pub enum DropAction {
    NewTransfer,
    AddToExisting,
    MergeDirectories,
    Cancel,
}

impl DropDialog {
    pub fn new(
        parent: Option<&gtk::Window>,
        device_name: &str,
        items: &[crate::ui::drag_drop::DroppedItem],
        existing_transfer: bool,
    ) -> Self {
        let dialog = Dialog::new();
        
        if let Some(p) = parent {
            dialog.set_transient_for(Some(p));
        }
        
        dialog.set_title(Some(&format!("Copiar a {}", device_name)));
        dialog.set_default_size(400, 300);
        
        let content_area = dialog.content_area();
        content_area.set_spacing(10);
        content_area.set_margin_all(15);
        
        // Información del dispositivo
        let device_box = Box::new(gtk::Orientation::Horizontal, 10);
        let device_icon = Image::from_icon_name("drive-harddisk-usb");
        let device_label = Label::new(Some(&format!("Dispositivo: {}", device_name)));
        
        device_box.append(&device_icon);
        device_box.append(&device_label);
        
        // Estadísticas de los items
        let stats = Self::calculate_stats(items);
        let stats_text = format!(
            "{} archivos, {} carpetas\nTotal: {} MB",
            stats.file_count,
            stats.dir_count,
            stats.total_size_mb
        );
        
        let stats_label = Label::new(Some(&stats_text));
        stats_label.set_xalign(0.0);
        
        // Opciones
        let options_box = Box::new(gtk::Orientation::Vertical, 5);
        
        let new_transfer_btn = Button::with_label("Nueva copia");
        new_transfer_btn.add_css_class("suggested-action");
        
        let add_to_existing_btn = if existing_transfer {
            let btn = Button::with_label("Añadir a copia actual");
            Some(btn)
        } else {
            None
        };
        
        let merge_dirs_btn = Button::with_label("Fusionar carpetas existentes");
        
        let cancel_btn = Button::with_label("Cancelar");
        
        // Checkbox para recordar elección
        let remember_check = CheckButton::with_label("Recordar esta elección para este dispositivo");
        
        // Organizar botones
        options_box.append(&new_transfer_btn);
        if let Some(btn) = &add_to_existing_btn {
            options_box.append(btn);
        }
        options_box.append(&merge_dirs_btn);
        options_box.append(&cancel_btn);
        options_box.append(&remember_check);
        
        // Añadir todo al diálogo
        content_area.append(&device_box);
        content_area.append(&stats_label);
        content_area.append(&options_box);
        
        let response = DropDialogResponse {
            action: DropAction::NewTransfer,
            remember_choice: false,
            options: crate::core::drag_drop::TransferOptions::default(),
        };
        
        let dialog_response = response.clone();
        
        // Conectar señales
        new_transfer_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(gtk::ResponseType::Accept);
            }
        });
        
        if let Some(btn) = add_to_existing_btn {
            btn.connect_clicked({
                let dialog = dialog.clone();
                move |_| {
                    dialog.emit_response(gtk::ResponseType::Other(1));
                }
            });
        }
        
        cancel_btn.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.emit_response(gtk::ResponseType::Cancel);
            }
        });
        
        Self { dialog, response: dialog_response }
    }
    
    pub async fn run(&self) -> Option<DropDialogResponse> {
        let response = self.dialog.run_future().await;
        
        match response {
            gtk::ResponseType::Accept => {
                Some(DropDialogResponse {
                    action: DropAction::NewTransfer,
                    remember_choice: self.response.remember_choice,
                    options: self.response.options.clone(),
                })
            }
            gtk::ResponseType::Other(1) => {
                Some(DropDialogResponse {
                    action: DropAction::AddToExisting,
                    remember_choice: self.response.remember_choice,
                    options: self.response.options.clone(),
                })
            }
            _ => None,
        }
    }
    
    fn calculate_stats(items: &[crate::ui::drag_drop::DroppedItem]) -> DropStats {
        let mut stats = DropStats {
            file_count: 0,
            dir_count: 0,
            total_size: 0,
            total_size_mb: 0,
        };
        
        for item in items {
            if item.is_dir {
                stats.dir_count += 1;
            } else {
                stats.file_count += 1;
            }
            stats.total_size += item.size;
        }
        
        stats.total_size_mb = stats.total_size / 1024 / 1024;
        stats
    }
}

struct DropStats {
    file_count: usize,
    dir_count: usize,
    total_size: u64,
    total_size_mb: u64,
}