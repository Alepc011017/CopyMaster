// src/core/transfer_manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct TransferManager {
    transfers: HashMap<Uuid, crate::core::device_queue::TransferJob>,
    active_transfers: Vec<Uuid>,
    options: crate::core::copy_engine::CopyOptions,
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            transfers: HashMap::new(),
            active_transfers: Vec::new(),
            options: crate::core::copy_engine::CopyOptions::default(),
            cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub async fn add_transfer(&mut self, transfer: crate::core::device_queue::TransferJob) {
        let id = Uuid::new_v4();
        // self.transfers.insert(id, transfer);
        // self.active_transfers.push(id);
    }
    
    pub async fn cancel_all(&mut self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        self.active_transfers.clear();
    }
    
    pub fn has_active_transfers(&self) -> bool {
        !self.active_transfers.is_empty()
    }
}