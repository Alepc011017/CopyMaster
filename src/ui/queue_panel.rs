// src/ui/queue_panel.rs
use gtk4::{prelude::*, Label, Box, ProgressBar, Button, Image};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct QueuePanel {
    pub container: gtk::Box,
    pub device_queues: Arc<Mutex<HashMap<String, crate::core::device_queue::DeviceQueue>>>,
}

impl QueuePanel {
    pub fn new() -> Self {
        let container = Box::new(gtk::Orientation::Vertical, 5);
        container.set_vexpand(true);
        
        Self {
            container,
            device_queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Añadir dispositivo a la vista de colas
    pub async fn add_device_queue(&self, device_name: String, queue: crate::core::device_queue::DeviceQueue) {
        let queue_widget = self.create_queue_widget(&device_name, &queue).await;
        self.container.append(&queue_widget);
        
        let mut queues = self.device_queues.lock().await;
        queues.insert(device_name, queue);
    }
    
    async fn create_queue_widget(&self, device_name: &str, queue: &crate::core::device_queue::DeviceQueue) -> gtk::Box {
        let device_box = Box::new(gtk::Orientation::Vertical, 5);
        device_box.set_margin_all(10);
        device_box.add_css_class("device-queue");
        
        // Cabecera del dispositivo
        let header_box = Box::new(gtk::Orientation::Horizontal, 10);
        
        let icon = Image::from_icon_name("drive-harddisk-usb");
        let name_label = Label::new(Some(device_name));
        name_label.set_xalign(0.0);
        
        let status_label = Label::new(Some(&format!("Estado: {:?}", queue.status)));
        let count_label = Label::new(Some(&format!("En cola: {}", queue.pending_transfers.len())));
        
        header_box.append(&icon);
        header_box.append(&name_label);
        header_box.append(&status_label);
        header_box.append(&count_label);
        
        // Barra de progreso de la transferencia actual
        let progress_box = Box::new(gtk::Orientation::Vertical, 5);
        
        if let Some(current) = &queue.current_transfer {
            let progress_info = self.create_transfer_progress(current).await;
            progress_box.append(&progress_info);
        }
        
        // Controles de la cola
        let controls_box = Box::new(gtk::Orientation::Horizontal, 5);
        
        let pause_btn = Button::with_label("Pausar");
        pause_btn.connect_clicked({
            let device_name = device_name.to_string();
            let queues = self.device_queues.clone();
            move |_| {
                tokio::spawn({
                    let device_name = device_name.clone();
                    let queues = queues.clone();
                    async move {
                        let mut queues_lock = queues.lock().await;
                        if let Some(queue) = queues_lock.get_mut(&device_name) {
                            // queue.pause().await;
                        }
                    }
                });
            }
        });
        
        let resume_btn = Button::with_label("Reanudar");
        resume_btn.connect_clicked({
            let device_name = device_name.to_string();
            let queues = self.device_queues.clone();
            move |_| {
                tokio::spawn({
                    let device_name = device_name.clone();
                    let queues = queues.clone();
                    async move {
                        let mut queues_lock = queues.lock().await;
                        if let Some(queue) = queues_lock.get_mut(&device_name) {
                            // queue.resume().await;
                        }
                    }
                });
            }
        });
        
        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.add_css_class("destructive-action");
        
        controls_box.append(&pause_btn);
        controls_box.append(&resume_btn);
        controls_box.append(&cancel_btn);
        
        // Lista de transferencias en cola
        let queue_list = self.create_queue_list(queue).await;
        
        device_box.append(&header_box);
        device_box.append(&progress_box);
        device_box.append(&controls_box);
        device_box.append(&queue_list);
        
        device_box
    }
    
    async fn create_transfer_progress(&self, transfer: &crate::core::device_queue::TransferJob) -> gtk::Box {
        let progress_box = Box::new(gtk::Orientation::Vertical, 5);
        
        let progress_bar = ProgressBar::new();
        progress_bar.set_fraction(
            transfer.copied_size as f64 / transfer.total_size.max(1) as f64
        );
        
        let info_label = Label::new(Some(&format!(
            "Copiando: {}/{} archivos ({} MB/{} MB)",
            transfer.completed_items,
            transfer.total_items,
            transfer.copied_size / 1024 / 1024,
            transfer.total_size / 1024 / 1024
        )));
        
        let speed_label = Label::new(Some("Calculando velocidad..."));
        
        progress_box.append(&progress_bar);
        progress_box.append(&info_label);
        progress_box.append(&speed_label);
        
        progress_box
    }
    
    async fn create_queue_list(&self, queue: &crate::core::device_queue::DeviceQueue) -> gtk::Box {
        let list_box = Box::new(gtk::Orientation::Vertical, 5);
        
        for transfer in queue.pending_transfers.iter() {
            let transfer_item = self.create_queue_item(transfer).await;
            list_box.append(&transfer_item);
        }
        
        list_box
    }
    
    async fn create_queue_item(&self, transfer: &crate::core::device_queue::TransferJob) -> gtk::Box {
        let item_box = Box::new(gtk::Orientation::Horizontal, 10);
        item_box.set_margin_all(5);
        item_box.add_css_class("transfer-item");
        
        let icon = Image::from_icon_name("document-send");
        let info_label = Label::new(Some(&format!(
            "Transferencia {} ({} items)",
            transfer.id,
            transfer.total_items
        )));
        
        item_box.append(&icon);
        item_box.append(&info_label);
        
        item_box
    }
}