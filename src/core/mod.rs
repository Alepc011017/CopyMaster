// src/core/mod.rs
pub mod autostart;
pub mod config;
pub mod copy_engine;
pub mod daemon;
pub mod device_detector;
pub mod device_queue;
pub mod drag_drop;
pub mod error_recovery;
pub mod extensions;
pub mod local_engine;
pub mod optimizer;
pub mod transfer_manager;
pub mod queue_manager;

// Re-exportar tipos comunes
pub use autostart::AutoStartManager;
pub use config::ConfigManager;
pub use device_queue::{DeviceQueue, TransferJob, TransferItem, TransferStatus, QueuePriority};
pub use drag_drop::{DroppedItem, TransferRequest, TransferOptions};
pub use error_recovery::{CopyError, RecoveryAction};
pub use copy_engine::{CopyOptions, CopyAlgorithm, CopyStats};