// src/core/daemon.rs
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

pub struct CopyMasterDaemon {
    is_running: bool,
    device_monitor: Arc<Mutex<crate::core::device_detector::DeviceMonitor>>,
    transfer_manager: Arc<Mutex<crate::core::transfer_manager::TransferManager>>,
    notification_channel: mpsc::Sender<DaemonNotification>,
}

#[derive(Debug)]
pub enum DaemonNotification {
    TransferStarted(String), // device_id
    TransferCompleted(String, crate::core::drag_drop::TransferResult),
    TransferError(String, String), // device_id, error
    DeviceConnected(String), // device_id
    DeviceDisconnected(String), // device_id
    NewDropEvent(DropEvent),
}

pub struct DropEvent {
    pub device_id: String,
    pub items: Vec<crate::core::drag_drop::DroppedItem>,
    pub timestamp: Instant,
}

impl CopyMasterDaemon {
    pub fn new() -> Self {
        let (notification_tx, _notification_rx) = mpsc::channel(100);
        
        Self {
            is_running: false,
            device_monitor: Arc::new(Mutex::new(crate::core::device_detector::DeviceMonitor::new())),
            transfer_manager: Arc::new(Mutex::new(crate::core::transfer_manager::TransferManager::new())),
            notification_channel: notification_tx,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_running = true;
        
        // Iniciar monitoreo de dispositivos
        let device_monitor = self.device_monitor.clone();
        let notification_tx = self.notification_channel.clone();
        
        tokio::spawn(async move {
            let mut monitor = device_monitor.lock().await;
            monitor.start_monitoring(notification_tx).await;
        });
        
        println!("CopyMaster daemon iniciado");
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        self.is_running = false;
        
        // Detener todas las transferencias
        let mut manager = self.transfer_manager.lock().await;
        manager.cancel_all().await;
        
        println!("CopyMaster daemon detenido");
    }
    
    pub async fn run_as_daemon(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Ejecutando CopyMaster en modo daemon...");
        
        // Iniciar servicio
        self.start().await?;
        
        // Mantener el daemon ejecutándose
        while self.is_running {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Verificar estado de transferencias
            let manager = self.transfer_manager.lock().await;
            let has_active_transfers = manager.has_active_transfers();
            
            // Aquí podríamos enviar notificaciones del sistema
            if has_active_transfers {
                // Actualizar icono en la bandeja, etc.
            }
        }
        
        Ok(())
    }
    
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}