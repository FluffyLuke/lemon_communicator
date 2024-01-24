use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::vec::Vec;

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use super::network::NETWORK_CHANGES;
use super::network::api::{NetworkChange, NetworkChangeType, NetworkStateMessage};

pub mod client_reqs;

lazy_static!{
    pub static ref KNOWN_CLIENTS: KnownClients = KnownClients::create();
}

struct KnownClients {
    clients: Arc<Mutex<Vec<Client>>>,
    id: AtomicU64
}

impl KnownClients {
    fn create() -> KnownClients {
        KnownClients {
            clients: Arc::new(Mutex::new(Vec::new())),
            id: AtomicU64::new(0),
        }
    }
    fn next_id(&self) -> u64 {
        self.id.fetch_add(1, Ordering::SeqCst)
    }
    pub async fn add(&self, client: Client) {
        let change: NetworkChange;
        {
            let mut locked_clients = self.clients.lock().await;
            change = NetworkChange::new(NetworkChangeType::JoinNetwork, Some(client.weak())).unwrap();
            locked_clients.push(client);
        }
        NETWORK_CHANGES.add(change);
    }
    pub async fn remove<T>(&self, client: &T) -> bool
    where T: GetId + GetWeak
    {
        let change;
        {
            let mut locked_clients = self.clients.lock().await;
            let position = locked_clients
                .iter()
                .position(&|locked_client: &Client| locked_client.id == client.get_id());
            if let None = position {
                return false;
            }
            change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(client.weak())).unwrap();
            locked_clients.remove(position.unwrap());
        }
        NETWORK_CHANGES.add(change);
        true
    }
    pub async fn get_network_state(&self) -> NetworkStateMessage {
        let locked_clients = self.clients.lock().await;
        NetworkStateMessage::new(&locked_clients)
    }
}

trait GetId {
    fn get_id(&self) -> u64;
}

pub trait GetWeak {
    fn weak(&self) -> WeakClient;
}

pub trait GetAddr {
    fn get_addr(&self) -> &std::net::SocketAddr;
}

#[derive(Debug)]
pub struct Client {
    pub id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
    pub stream: tokio::io::BufReader<tokio::net::TcpStream>,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String, stream: BufReader<TcpStream>) -> Client {
        Client {
            id: KNOWN_CLIENTS.next_id(),
            addr,
            name,
            stream,
        }
    }
}

impl GetId for Client {
    fn get_id(&self) -> u64 {
        self.id
    }
}
impl GetWeak for Client {
    fn weak(&self) -> WeakClient {
        WeakClient {
            id: self.id,
            addr: self.addr,
            name: self.name,
        }
    }
}
impl GetAddr for Client {
    fn get_addr(&self) -> &std::net::SocketAddr {
        &self.addr
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct WeakClient {
    id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
}

impl GetId for WeakClient {
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl GetWeak for WeakClient {
    fn weak(&self) -> WeakClient {
        *self
    }
}

impl GetAddr for WeakClient {
    fn get_addr(&self) -> &std::net::SocketAddr {
        &self.addr
    }
}