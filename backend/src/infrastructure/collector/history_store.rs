use crate::domain::entities::CollectorSnapshot;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global in-memory storage for collector snapshots, keyed by normalized target URL.
static HISTORY_STORE: OnceLock<Mutex<HashMap<String, Vec<CollectorSnapshot>>>> = OnceLock::new();

fn get_store() -> &'static Mutex<HashMap<String, Vec<CollectorSnapshot>>> {
    HISTORY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Saves a snapshot and returns true if it succeeded.
pub fn save_snapshot(target: &str, snapshot: CollectorSnapshot) -> bool {
    let store = get_store();
    if let Ok(mut map) = store.lock() {
        let entry = map.entry(target.to_string()).or_insert_with(Vec::new);
        entry.push(snapshot);
        true
    } else {
        false
    }
}

/// Retrieves the most recent previous snapshot for a target, if any.
pub fn get_previous_snapshot(target: &str) -> Option<CollectorSnapshot> {
    let store = get_store();
    if let Ok(map) = store.lock() {
        if let Some(history) = map.get(target) {
            return history.last().cloned();
        }
    }
    None
}
