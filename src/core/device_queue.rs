use std::collections::{VecDeque, BinaryHeap};
use std::path::PathBuf;
use std::time::{Instant, Duration};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum QueuePriority {
    Background,
    Normal,
    Interactive,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DeviceQueue {
    pub device_path: PathBuf,
    pub device_name: String,
    pub current_transfer: Option<Arc<TransferJob>>,
    pub pending_transfers: VecDeque<Arc<TransferJob>>,
    pub completed_transfers: VecDeque<CompletedTransfer>,
    pub priority_queue: BinaryHeap<PrioritizedTransfer>,
    pub status: QueueStatus,
    pub created_at: Instant,
    pub stats: QueueStatistics,
}

#[derive(Debug, Clone)]
pub struct TransferJob {
    pub id: u64,
    pub root_items: Vec<TransferItem>,
    pub destination: PathBuf,
    pub status: TransferStatus,
    pub progress_sender: mpsc::Sender<TransferProgress>,
    pub progress_receiver: mpsc::Receiver<TransferProgress>,
    pub created_at: Instant,
    pub priority: QueuePriority,
    pub total_items: usize,
    pub completed_items: usize,
    pub total_size: u64,
    pub copied_size: u64,
    // NUEVO: Configuración de conflictos para esta transferencia
    pub conflict_settings: crate::core::copy_engine::RuntimeConflictSettings,
    pub transfer_name: String, // Nombre de la transferencia para mostrar en diálogos
    pub ui_conflict_channel: Option<tokio::sync::mpsc::Sender<crate::ui::conflict_dialog::ConflictDialogRequest>>,
}

#[derive(Debug, Clone)]
pub struct TransferItem {
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub item_type: ItemType,
    pub size: u64,
    pub children: Vec<TransferItem>,
    pub status: ItemTransferStatus,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub enum ItemTransferStatus {
    Pending,
    CreatingDir,
    Copying,
    Verifying,
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum TransferStatus {
    Queued,
    Preparing,
    Copying,
    Verifying,
    Paused,
    Completed,
    Error,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct CompletedTransfer {
    pub job: Arc<TransferJob>,
    pub result: crate::core::drag_drop::TransferResult,
    pub completed_at: Instant,
}

#[derive(Debug, Clone)]
pub struct QueueStatistics {
    pub total_transfers: usize,
    pub successful_transfers: usize,
    pub failed_transfers: usize,
    pub total_bytes: u64,
    pub total_duration: Duration,
}

impl Default for QueueStatistics {
    fn default() -> Self {
        Self {
            total_transfers: 0,
            successful_transfers: 0,
            failed_transfers: 0,
            total_bytes: 0,
            total_duration: Duration::from_secs(0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransferProgress {
    FileStarted {
        source: PathBuf,
        destination: PathBuf,
        size: u64,
    },
    FileCompleted {
        source: PathBuf,
        destination: PathBuf,
        size: u64,
        duration: Duration,
    },
    DirectoryCreated {
        path: PathBuf,
        item_count: usize,
    },
    AddItems(Vec<TransferItem>),
    // NUEVO: Conflicto detectado
    ConflictDetected {
        source: PathBuf,
        destination: PathBuf,
    },
    // NUEVO: Resolución de conflicto
    ConflictResolved {
        source: PathBuf,
        destination: PathBuf,
        action: String,
    },
}
#[derive(Debug, Clone, Eq)]
pub struct PrioritizedTransfer {
    pub priority: QueuePriority,
    pub created_at: Instant,
    pub job: Arc<TransferJob>,
}

impl Ord for PrioritizedTransfer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => other.created_at.cmp(&self.created_at),
            other => other,
        }
    }
}

impl PartialOrd for PrioritizedTransfer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrioritizedTransfer {
    fn eq(&self, other: &Self) -> bool {
        self.job.id == other.job.id
    }
}

// NUEVO ENUM: Resultado de la resolución de conflicto
#[derive(Debug, Clone)]
pub enum ConflictResolutionResult {
    Overwrite,
    Skip,
    RenameNew,
    RenameOld,
    Ask, // Necesita intervención del usuario
    Cancelled, // El usuario canceló
}

impl TransferJob {
    // NUEVO MÉTODO: Manejar conflicto de archivos
    pub async fn handle_conflict(
        &mut self,
        source: &PathBuf,
        dest: &PathBuf,
        config: &crate::core::config::AppConfig,
    ) -> ConflictResolutionResult {
        // 1. Verificar si ya tenemos una decisión para "siempre"
        if self.conflict_settings.overwrite_all {
            return ConflictResolutionResult::Overwrite;
        }
        if self.conflict_settings.skip_all {
            return ConflictResolutionResult::Skip;
        }
        
        // 2. Verificar configuración global
        if !config.conflict_resolution.ask_for_confirmation {
            return match config.conflict_resolution.default_action {
                crate::core::config::ConflictAction::Overwrite => ConflictResolutionResult::Overwrite,
                crate::core::config::ConflictAction::Skip => ConflictResolutionResult::Skip,
                crate::core::config::ConflictAction::RenameNew => ConflictResolutionResult::RenameNew,
                crate::core::config::ConflictAction::RenameOld => ConflictResolutionResult::RenameOld,
                _ => ConflictResolutionResult::Ask,
            };
        }
        
        // 3. Verificar acción actual de esta transferencia
        if self.conflict_settings.current_action != crate::core::config::ConflictAction::Ask {
            return match self.conflict_settings.current_action {
                crate::core::config::ConflictAction::Overwrite => ConflictResolutionResult::Overwrite,
                crate::core::config::ConflictAction::Skip => ConflictResolutionResult::Skip,
                crate::core::config::ConflictAction::RenameNew => ConflictResolutionResult::RenameNew,
                crate::core::config::ConflictAction::RenameOld => ConflictResolutionResult::RenameOld,
                _ => ConflictResolutionResult::Ask,
            };
        }
        
        // 4. Si tenemos canal de UI, pedir al usuario
        if let Some(channel) = &self.ui_conflict_channel {
            let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
            
            let request = crate::ui::conflict_dialog::ConflictDialogRequest {
                source: source.clone(),
                destination: dest.clone(),
                transfer_name: self.transfer_name.clone(),
                response_sender,
            };
            
            // Enviar solicitud a la UI
            if channel.send(request).await.is_ok() {
                // Esperar respuesta (con timeout)
                match tokio::time::timeout(std::time::Duration::from_secs(30), response_receiver).await {
                    Ok(Ok(response)) => return response,
                    Ok(Err(_)) => return ConflictResolutionResult::Ask,
                    Err(_) => return ConflictResolutionResult::Skip, // Timeout
                }
            }
        }
        
        // 5. Fallback: cancelar
        ConflictResolutionResult::Cancelled
    }
    
    // NUEVO MÉTODO: Actualizar configuración de conflictos
    pub fn update_conflict_settings(&mut self, action: crate::core::config::ConflictAction, remember_for_transfer: bool) {
        self.conflict_settings.update_from_action(&action);
        
        if remember_for_transfer {
            match action {
                crate::core::config::ConflictAction::OverwriteAll => {
                    self.conflict_settings.overwrite_all = true;
                }
                crate::core::config::ConflictAction::SkipAll => {
                    self.conflict_settings.skip_all = true;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueueStatus {
    Idle,
    Active,
    Paused,
    Stopped,
}

impl DeviceQueue {
    pub fn new(device_path: PathBuf) -> Self {
        let device_name = device_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
            
        Self {
            device_path,
            device_name,
            current_transfer: None,
            pending_transfers: VecDeque::new(),
            completed_transfers: VecDeque::new(),
            priority_queue: BinaryHeap::new(),
            status: QueueStatus::Idle,
            created_at: Instant::now(),
            stats: QueueStatistics::default(),
        }
    }
    
    // ... resto de métodos existentes ...
}