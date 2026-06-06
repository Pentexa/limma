use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct ScanTaskHandle {
    pub is_cancelled: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
}

pub struct ScanController {
    active_tasks: RwLock<HashMap<Uuid, ScanTaskHandle>>,
}

impl ScanController {
    pub fn new() -> Self {
        Self {
            active_tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_scan(&self, scan_id: Uuid) -> ScanTaskHandle {
        let handle = ScanTaskHandle {
            is_cancelled: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
        };
        
        let mut tasks = self.active_tasks.write().await;
        tasks.insert(scan_id, handle.clone());
        
        handle
    }

    pub async fn unregister_scan(&self, scan_id: &Uuid) {
        let mut tasks = self.active_tasks.write().await;
        tasks.remove(scan_id);
    }

    pub async fn pause_scan(&self, scan_id: &Uuid) -> Result<(), String> {
        let tasks = self.active_tasks.read().await;
        if let Some(handle) = tasks.get(scan_id) {
            handle.is_paused.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Scan not found or not currently active".to_string())
        }
    }

    pub async fn resume_scan(&self, scan_id: &Uuid) -> Result<(), String> {
        let tasks = self.active_tasks.read().await;
        if let Some(handle) = tasks.get(scan_id) {
            handle.is_paused.store(false, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Scan not found or not currently active".to_string())
        }
    }

    pub async fn cancel_scan(&self, scan_id: &Uuid) -> Result<(), String> {
        let tasks = self.active_tasks.read().await;
        if let Some(handle) = tasks.get(scan_id) {
            handle.is_cancelled.store(true, Ordering::SeqCst);
            handle.is_paused.store(false, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Scan not found or not currently active".to_string())
        }
    }
}
