// src/core/queue_manager.rs
use std::collections::{VecDeque, BinaryHeap};
use std::cmp::Reverse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TransferQueue {
    pending: BinaryHeap<QueuedTransfer>,
    active: Vec<crate::core::device_queue::TransferJob>,
    completed: VecDeque<crate::core::device_queue::CompletedTransfer>,
    max_concurrent: usize,
}

#[derive(Debug, Clone)]
pub struct QueuedTransfer {
    pub id: uuid::Uuid,
    pub priority: Priority,
    pub estimated_size: u64,
    pub queued_at: std::time::Instant,
    pub source: std::path::PathBuf,
    pub destination: std::path::PathBuf,
}

impl Ord for QueuedTransfer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => 
                self.estimated_size.cmp(&other.estimated_size),
            other => other,
        }
    }
}

impl PartialOrd for QueuedTransfer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueuedTransfer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for QueuedTransfer {}

impl TransferQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: BinaryHeap::new(),
            active: Vec::new(),
            completed: VecDeque::new(),
            max_concurrent,
        }
    }
    
    pub fn schedule_smart(&mut self, transfers: Vec<QueuedTransfer>) {
        let mut transfers_by_dir: std::collections::HashMap<std::path::PathBuf, Vec<QueuedTransfer>> = 
            std::collections::HashMap::new();
        
        for transfer in transfers {
            let dir = transfer.source.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf();
            
            transfers_by_dir.entry(dir).or_default().push(transfer);
        }
        
        for (_, mut dir_transfers) in transfers_by_dir {
            dir_transfers.sort_by_key(|t| t.source.clone());
            for transfer in dir_transfers {
                self.pending.push(transfer);
            }
        }
    }
    
    pub fn next_optimized(&mut self) -> Option<QueuedTransfer> {
        self.pending.pop()
    }
}