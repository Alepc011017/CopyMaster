// src/core/error_recovery.rs
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CopyError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Disk full")]
    DiskFull,
    
    #[error("File locked")]
    FileLocked,
    
    #[error("Network error")]
    NetworkError,
    
    #[error("Hash mismatch")]
    HashMismatch,
    
    #[error("Cancelled")]
    Cancelled,
    
    #[error("Paused")]
    Paused,
    
    #[error("Invalid path")]
    InvalidPath,
    
    #[error("Cross device link")]
    CrossDeviceLink,
}

impl CopyError {
    pub fn can_retry(&self) -> bool {
        matches!(self, CopyError::FileLocked | CopyError::NetworkError | CopyError::HashMismatch)
    }
    
    pub fn suggested_action(&self) -> RecoveryAction {
        match self {
            CopyError::DiskFull => RecoveryAction::FreeSpace,
            CopyError::PermissionDenied => RecoveryAction::RequestElevation,
            CopyError::FileLocked => RecoveryAction::UnlockOrSkip,
            CopyError::CrossDeviceLink => RecoveryAction::UseCopy,
            _ => RecoveryAction::RetryOrSkip,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry,
    Skip,
    RetryAll,
    SkipAll,
    FreeSpace,
    RequestElevation,
    UnlockOrSkip,
    UseCopy,
}

pub struct ErrorRecovery {
    max_retries: usize,
    retry_delay: std::time::Duration,
    auto_recover: bool,
}

impl ErrorRecovery {
    pub fn new(max_retries: usize, retry_delay: std::time::Duration, auto_recover: bool) -> Self {
        Self {
            max_retries,
            retry_delay,
            auto_recover,
        }
    }
    
    pub async fn handle_error(
        &self,
        error: &CopyError,
        transfer: &crate::core::device_queue::TransferItem,
    ) -> Result<RecoveryAction, CopyError> {
        if !error.can_retry() {
            return Err(error.clone());
        }
        
        // Intentar recuperación automática para ciertos errores
        match error {
            CopyError::FileLocked => {
                if self.auto_recover {
                    self.unlock_file(&transfer.source_path).await?;
                    return Ok(RecoveryAction::Retry);
                }
            }
            CopyError::DiskFull => {
                if self.auto_recover {
                    self.clean_temp_files().await?;
                    return Ok(RecoveryAction::Retry);
                }
            }
            _ => {}
        }
        
        Ok(RecoveryAction::Retry)
    }
    
    async fn unlock_file(&self, path: &std::path::PathBuf) -> Result<(), CopyError> {
        // Implementar lógica de desbloqueo
        // Por ahora, solo retornamos Ok
        Ok(())
    }
    
    async fn clean_temp_files(&self) -> Result<(), CopyError> {
        // Implementar limpieza de archivos temporales
        Ok(())
    }
}