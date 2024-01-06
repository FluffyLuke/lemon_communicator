use std::ops::Index;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::vec::Vec;

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

use super::network::NETWORK_CHANGES;
use super::network::api::{NetworkChange, NetworkChangeType};

pub mod client_reqs;

lazy_static!{
    pub static ref KNOWN_CLIENTS: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize, std::cmp::Eq, PartialEq)]
pub struct Client {
    pub id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String) -> Client {
        Client {
            id: ID.fetch_add(1, Ordering::SeqCst),
            addr,
            name,
        }
    }
}

async fn remove_client_from_clients(client: &Client) -> Option<()> {
    let mut locked_clients = KNOWN_CLIENTS.lock().await;
    let mut locked_changes = NETWORK_CHANGES.lock().await;
    let position = locked_clients.iter().position(&|locked_client: &Client| locked_client.id == client.id)?;
    let change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(&client)).unwrap();
    locked_clients.remove(position);
    locked_changes.changes.append(&mut vec![change]);
    Some(())
}

async fn remove_client_from_clients_by_addr(addr: std::net::SocketAddr) -> Option<()>{
    let mut locked_clients = KNOWN_CLIENTS.lock().await;
    let mut locked_changes = NETWORK_CHANGES.lock().await;
    let position = locked_clients.iter().position(&|locked_client: &Client| locked_client.addr == addr)?;
    let client = locked_clients.index(position);
    let change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(&client)).unwrap();
    locked_clients.remove(position);
    locked_changes.changes.append(&mut vec![change]);
    Some(())
}