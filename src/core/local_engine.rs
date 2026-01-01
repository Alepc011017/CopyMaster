use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;

#[async_trait]
pub trait CopySystem: Send + Sync {
    async fn copy(&self, source: &PathBuf, dest: &PathBuf) -> Result<crate::core::copy_engine::CopyStats, crate::core::error_recovery::CopyError>;
    async fn pause(&self) -> Result<(), crate::core::error_recovery::CopyError>;
    async fn resume(&self) -> Result<(), crate::core::error_recovery::CopyError>;
    async fn cancel(&self) -> Result<(), crate::core::error_recovery::CopyError>;
    fn get_stats(&self) -> crate::core::copy_engine::CopyStats;
    fn get_options(&self) -> &crate::core::copy_engine::CopyOptions;
}

pub struct LocalCopySystem {
    engine: LocalCopyEngine,
    device_manager: crate::core::device_detector::DeviceMonitor,
    options: crate::core::copy_engine::CopyOptions,
}

pub struct LocalCopyEngine {
    options: crate::core::copy_engine::CopyOptions,
}

impl LocalCopyEngine {
    pub fn new(options: crate::core::copy_engine::CopyOptions) -> Self {
        Self { options }
    }
    
    pub async fn copy_file_optimized(
        &self,
        source: &PathBuf,
        dest: &PathBuf,
        device_info: &crate::core::device_detector::DeviceInfo,
    ) -> Result<crate::core::copy_engine::CopyStats, crate::core::error_recovery::CopyError> {
        // Implementar copia optimizada
        match std::fs::copy(source, dest) {
            Ok(_) => {
                let stats = crate::core::copy_engine::CopyStats::new();
                Ok(stats)
            }
            Err(e) => Err(crate::core::error_recovery::CopyError::Io(e.to_string())),
        }
    }
    
    // NUEVO MÉTODO: Copiar con manejo de conflictos
    pub async fn copy_with_conflict_handling(
        &self,
        source: &PathBuf,
        dest: &PathBuf,
        job: &crate::core::device_queue::TransferJob,
        config: &crate::core::config::AppConfig,
    ) -> Result<(), crate::core::error_recovery::CopyError> {
        // Verificar si el archivo de destino ya existe
        if dest.exists() {
            // Clonar job para poder mutarlo
            let mut job_mut = job.clone();
            
            // Notificar conflicto
            let _ = job.progress_sender.send(
                crate::core::device_queue::TransferProgress::ConflictDetected {
                    source: source.clone(),
                    destination: dest.clone(),
                }
            ).await;
            
            // Manejar conflicto
            let result = job_mut.handle_conflict(source, dest, config).await;
            
            match result {
                crate::core::device_queue::ConflictResolutionResult::Overwrite => {
                    // Sobrescribir archivo
                    std::fs::copy(source, dest)
                        .map_err(|e| crate::core::error_recovery::CopyError::Io(e.to_string()))?;
                    
                    // Notificar resolución
                    let _ = job.progress_sender.send(
                        crate::core::device_queue::TransferProgress::ConflictResolved {
                            source: source.clone(),
                            destination: dest.clone(),
                            action: "Sobrescrito".to_string(),
                        }
                    ).await;
                }
                crate::core::device_queue::ConflictResolutionResult::Skip => {
                    // Saltar archivo
                    let _ = job.progress_sender.send(
                        crate::core::device_queue::TransferProgress::ConflictResolved {
                            source: source.clone(),
                            destination: dest.clone(),
                            action: "Saltado".to_string(),
                        }
                    ).await;
                    return Ok(());
                }
                crate::core::device_queue::ConflictResolutionResult::RenameNew => {
                    // Renombrar el nuevo archivo
                    let new_dest = self.generate_unique_filename(dest).await;
                    std::fs::copy(source, &new_dest)
                        .map_err(|e| crate::core::error_recovery::CopyError::Io(e.to_string()))?;
                    
                    let _ = job.progress_sender.send(
                        crate::core::device_queue::TransferProgress::ConflictResolved {
                            source: source.clone(),
                            destination: new_dest.clone(),
                            action: "Renombrado nuevo".to_string(),
                        }
                    ).await;
                }
                crate::core::device_queue::ConflictResolutionResult::RenameOld => {
                    // Renombrar el archivo existente
                    let old_dest = self.generate_unique_filename(dest).await;
                    std::fs::rename(dest, &old_dest)
                        .map_err(|e| crate::core::error_recovery::CopyError::Io(e.to_string()))?;
                    
                    // Copiar el nuevo archivo
                    std::fs::copy(source, dest)
                        .map_err(|e| crate::core::error_recovery::CopyError::Io(e.to_string()))?;
                    
                    let _ = job.progress_sender.send(
                        crate::core::device_queue::TransferProgress::ConflictResolved {
                            source: source.clone(),
                            destination: dest.clone(),
                            action: "Renombrado antiguo".to_string(),
                        }
                    ).await;
                }
                crate::core::device_queue::ConflictResolutionResult::Ask => {
                    // Necesita intervención del usuario
                    return Err(crate::core::error_recovery::CopyError::Cancelled);
                }
                crate::core::device_queue::ConflictResolutionResult::Cancelled => {
                    // El usuario canceló
                    return Err(crate::core::error_recovery::CopyError::Cancelled);
                }
            }
        } else {
            // No hay conflicto, copiar normalmente
            std::fs::copy(source, dest)
                .map_err(|e| crate::core::error_recovery::CopyError::Io(e.to_string()))?;
        }
        
        Ok(())
    }
    
    // NUEVO MÉTODO: Generar nombre único para archivos
    async fn generate_unique_filename(&self, original: &PathBuf) -> PathBuf {
        let mut counter = 1;
        
        loop {
            let new_name = if let Some(parent) = original.parent() {
                if let Some(stem) = original.file_stem() {
                    if let Some(ext) = original.extension() {
                        parent.join(format!("{} ({}).{}", 
                            stem.to_string_lossy(), 
                            counter, 
                            ext.to_string_lossy()))
                    } else {
                        parent.join(format!("{} ({})", 
                            stem.to_string_lossy(), 
                            counter))
                    }
                } else {
                    original.with_file_name(format!("file ({})", counter))
                }
            } else {
                PathBuf::from(format!("file ({})", counter))
            };
            
            if !new_name.exists() {
                return new_name;
            }
            
            counter += 1;
            
            // Limitar para evitar bucle infinito
            if counter > 1000 {
                break;
            }
        }
        
        // Fallback: añadir timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if let Some(parent) = original.parent() {
            if let Some(stem) = original.file_stem() {
                if let Some(ext) = original.extension() {
                    parent.join(format!("{}_copy_{}.{}", 
                        stem.to_string_lossy(), 
                        timestamp, 
                        ext.to_string_lossy()))
                } else {
                    parent.join(format!("{}_copy_{}", 
                        stem.to_string_lossy(), 
                        timestamp))
                }
            } else {
                parent.join(format!("copy_{}", timestamp))
            }
        } else {
            PathBuf::from(format!("copy_{}", timestamp))
        }
    }
}