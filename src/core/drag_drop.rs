// src/core/drag_drop.rs
use std::path::PathBuf;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedItem {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub item_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DropAction {
    NewTransfer,
    AddToExisting,
    MergeDirectories,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: u64,
    pub source_items: Vec<DroppedItem>,
    pub destination: PathBuf,
    pub device_id: String,
    pub created_at: Instant,
    pub options: TransferOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOptions {
    pub priority: Priority,
    pub verify_after_copy: bool,
    pub preserve_attributes: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            priority: Priority::Normal,
            verify_after_copy: true,
            preserve_attributes: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferResult {
    pub transfer_id: u64,
    pub status: crate::core::device_queue::TransferStatus,
    pub total_bytes: u64,
    pub files_copied: usize,
    pub directories_created: usize,
    pub duration: std::time::Duration,
    pub errors: Vec<String>,
}

impl TransferResult {
    pub fn new(transfer_id: u64) -> Self {
        Self {
            transfer_id,
            status: crate::core::device_queue::TransferStatus::Queued,
            total_bytes: 0,
            files_copied: 0,
            directories_created: 0,
            duration: std::time::Duration::from_secs(0),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransferError {
    IoError(String),
    Cancelled,
    Paused,
}