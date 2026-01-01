use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CopyAlgorithm {
    /// Copia estándar del sistema
    Standard,
    /// Copia en paralelo por chunks
    ParallelChunks,
    /// Copia con verificación en tiempo real
    Verified,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    Overwrite,
    RenameNew,
    RenameOld,
    Skip,
    AskUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOptions {
    pub algorithm: CopyAlgorithm,
    pub buffer_size: usize,        // 64KB por defecto
    pub max_threads: usize,        // Núcleos * 2
    pub verify_after_copy: bool,
    pub conflict_resolution: ConflictResolution,
    pub preserve_attributes: bool, // permisos, timestamps
    pub sparse_files: bool,        // Archivos sparse
    pub sync_io: bool,            // O_SYNC para mayor seguridad
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            algorithm: CopyAlgorithm::ParallelChunks,
            buffer_size: 65536,
            max_threads: num_cpus::get() * 2,
            verify_after_copy: true,
            conflict_resolution: ConflictResolution::AskUser,
            preserve_attributes: true,
            sparse_files: true,
            sync_io: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CopyStats {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: f64,
    pub elapsed_time: std::time::Duration,
    pub estimated_remaining: std::time::Duration,
    pub files_copied: usize,
    pub files_total: usize,
    pub errors: Vec<String>,
}

impl CopyStats {
    pub fn new() -> Self {
        Self {
            bytes_transferred: 0,
            total_bytes: 0,
            speed_bps: 0.0,
            elapsed_time: std::time::Duration::from_secs(0),
            estimated_remaining: std::time::Duration::from_secs(0),
            files_copied: 0,
            files_total: 0,
            errors: Vec::new(),
        }
    }
}

// NUEVA ESTRUCTURA: Configuración de conflictos en tiempo de ejecución
#[derive(Debug, Clone)]
pub struct RuntimeConflictSettings {
    pub current_action: crate::core::config::ConflictAction,
    pub overwrite_all: bool,
    pub skip_all: bool,
    pub ask_for_each: bool,
}

impl RuntimeConflictSettings {
    pub fn new() -> Self {
        Self {
            current_action: crate::core::config::ConflictAction::Ask,
            overwrite_all: false,
            skip_all: false,
            ask_for_each: true,
        }
    }
    
    pub fn update_from_action(&mut self, action: &crate::core::config::ConflictAction) {
        match action {
            crate::core::config::ConflictAction::OverwriteAll => {
                self.overwrite_all = true;
                self.skip_all = false;
                self.ask_for_each = false;
                self.current_action = crate::core::config::ConflictAction::Overwrite;
            }
            crate::core::config::ConflictAction::SkipAll => {
                self.skip_all = true;
                self.overwrite_all = false;
                self.ask_for_each = false;
                self.current_action = crate::core::config::ConflictAction::Skip;
            }
            _ => {
                self.current_action = action.clone();
            }
        }
    }
    
    pub fn should_ask(&self) -> bool {
        self.ask_for_each && 
        !self.overwrite_all && 
        !self.skip_all &&
        self.current_action == crate::core::config::ConflictAction::Ask
    }
}