// src/ui/drag_drop.rs
use gtk4::{gio, glib, gdk, prelude::*};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct DroppedItem {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub item_count: Option<usize>,
}

#[derive(Clone)]
pub struct TransferRequest {
    pub id: u64,
    pub source_items: Vec<DroppedItem>,
    pub destination: PathBuf,
    pub device_id: String,
    pub created_at: std::time::Instant,
    pub options: crate::core::drag_drop::TransferOptions,
}

pub struct DragDropManager {
    window: gtk::Window,
    drop_areas: HashMap<String, gtk::DropTarget>,
    pending_transfers: Arc<Mutex<Vec<TransferRequest>>>,
    device_queues: Arc<Mutex<HashMap<String, crate::core::device_queue::DeviceQueue>>>,
}

impl DragDropManager {
    pub fn new(window: gtk::Window) -> Self {
        Self {
            window,
            drop_areas: HashMap::new(),
            pending_transfers: Arc::new(Mutex::new(Vec::new())),
            device_queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Configurar área de drop para un dispositivo específico
    pub fn setup_device_drop_area(&mut self, device_path: &PathBuf, device_name: &str) {
        let drop_target = gtk::DropTarget::new(
            gio::File::static_type(),
            gdk::DragAction::COPY
        );
        
        let device_path = device_path.clone();
        let device_name = device_name.to_string();
        let pending_transfers = self.pending_transfers.clone();
        let device_queues = self.device_queues.clone();
        
        drop_target.connect_drop(move |_, value, _, _| {
            if let Some(file_list) = value.get::<gio::ListModel>() {
                let items = Self::extract_dropped_items(&file_list);
                
                // Mostrar diálogo de confirmación
                let response = Self::show_drop_dialog(&device_name, &items);
                
                match response {
                    DropResponse::NewTransfer => {
                        let transfer = TransferRequest {
                            id: rand::random::<u64>(),
                            source_items: items,
                            destination: device_path.clone(),
                            device_id: device_name.clone(),
                            created_at: std::time::Instant::now(),
                            options: crate::core::drag_drop::TransferOptions::default(),
                        };
                        
                        // Añadir a cola del dispositivo
                        tokio::spawn({
                            let device_queues = device_queues.clone();
                            let transfer = transfer.clone();
                            async move {
                                let mut queues = device_queues.lock().await;
                                let queue = queues.entry(device_name.clone())
                                    .or_insert_with(|| crate::core::device_queue::DeviceQueue::new(device_path.clone()));
                                    
                                queue.add_transfer(transfer).await;
                            }
                        });
                    }
                    DropResponse::AddToExisting => {
                        // Buscar transferencia existente para este dispositivo
                        tokio::spawn({
                            let device_queues = device_queues.clone();
                            let device_name = device_name.clone();
                            let items = items.clone();
                            async move {
                                let mut queues = device_queues.lock().await;
                                if let Some(queue) = queues.get_mut(&device_name) {
                                    queue.add_items_to_current_transfer(items).await;
                                }
                            }
                        });
                    }
                    DropResponse::Cancel => {}
                }
                
                true
            } else {
                false
            }
        });
        
        self.window.add_controller(drop_target);
        self.drop_areas.insert(device_name.to_string(), drop_target);
    }
    
    fn extract_dropped_items(file_list: &gio::ListModel) -> Vec<DroppedItem> {
        let mut items = Vec::new();
        
        for i in 0..file_list.n_items() {
            if let Some(file) = file_list.item(i).and_then(|obj| obj.downcast::<gio::File>().ok()) {
                if let Ok(path) = file.path() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        let item = DroppedItem {
                            path: path.clone(),
                            size: metadata.len(),
                            is_dir: metadata.is_dir(),
                            item_count: if metadata.is_dir() {
                                Self::count_dir_items(&path).ok()
                            } else {
                                None
                            },
                        };
                        items.push(item);
                    }
                }
            }
        }
        
        items
    }
    
    fn count_dir_items(path: &PathBuf) -> Result<usize, std::io::Error> {
        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let _ = entry?;
            count += 1;
        }
        Ok(count)
    }
    
    fn show_drop_dialog(device_name: &str, items: &[DroppedItem]) -> DropResponse {
        // En una implementación real, mostrarías un diálogo GTK
        // Por ahora, simulamos la lógica
        
        let total_size: u64 = items.iter().map(|i| i.size).sum();
        let dir_count = items.iter().filter(|i| i.is_dir).count();
        let file_count = items.len() - dir_count;
        
        println!("Soltando en {}: {} archivos, {} carpetas, {} MB",
                 device_name, file_count, dir_count, total_size / 1024 / 1024);
        
        // Verificar si hay transferencia activa para este dispositivo
        // Si la hay, preguntar si añadir o nueva transferencia
        DropResponse::NewTransfer // Por defecto
    }
}

enum DropResponse {
    NewTransfer,
    AddToExisting,
    Cancel,
}