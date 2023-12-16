use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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