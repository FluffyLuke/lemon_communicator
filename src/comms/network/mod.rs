use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use self::api::{NetworkChange, NetworkChangesMessage};


pub mod api;

lazy_static! {
    pub static ref NETWORK_CHANGES: NetworkChanges = NetworkChanges::new();
}

pub struct NetworkChanges {
    changes: Arc<Mutex<NetworkChangesMessage>>,
}

impl NetworkChanges {
    pub fn new() -> NetworkChanges{
        NetworkChanges {
            changes: Arc::new(Mutex::new(NetworkChangesMessage::new(vec![])))
        }
    }
    pub async fn add(&self, network_change: NetworkChange) {
        self.changes.lock().await.changes.push(network_change);
    }
    pub async fn add_multiple(&self, network_changes: Vec<NetworkChange>) {
        let mut locked_changes = self.changes.lock().await;
        for change in network_changes {
            locked_changes.changes.push(change)
        }
    }
    pub async fn get_changes_json(&self) -> Option<String> {
        let locked_changes = self.changes.lock().await;
        if locked_changes.changes.is_empty() {
            return None
        }
        Some(serde_json::to_string(&*locked_changes).unwrap())
    }
    pub async fn reset_changes(&self) {
        self.changes.lock().await.changes.clear();
    }
}