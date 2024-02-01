use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::vec::Vec;

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::comms::network::api::{GenericMessage, MessageType, Status};

use super::network::NETWORK_CHANGES;
use super::network::api::{NetworkChange, NetworkChangeType, NetworkStateMessage};

pub mod client_reqs;
pub mod client_handler;

lazy_static!{
    pub static ref KNOWN_CLIENTS: KnownClients = KnownClients::create();
}

#[derive(Debug, Clone)]
pub enum ServerActions {
    UpdateClient(String),
}


pub struct KnownClients {
    // TODO Make better api for accessing clients
    pub registered_clients: Arc<RwLock<Vec<RegisteredClient>>>,
    id: AtomicU64
}

impl KnownClients {
    fn create() -> KnownClients {
        KnownClients {
            registered_clients: Arc::new(RwLock::new(Vec::new())),
            id: AtomicU64::new(0),
        }
    }
    async fn to_weak(&self) -> Vec<WeakClient> {
        let locked_clients = self.registered_clients.read().await;
        let mut weak_clients = vec![];
        for client in locked_clients.iter() {
            weak_clients.push(client.weak())
        }
        return weak_clients
    }

    fn next_id(&self) -> u64 {
        self.id.fetch_add(1, Ordering::SeqCst)
    }
    pub async fn add(&self, client: RegisteredClient) {
        let change: NetworkChange;
        {
            let mut locked_clients = self.registered_clients.write().await;
            change = NetworkChange::new(NetworkChangeType::JoinNetwork, Some(client.weak())).unwrap();
            locked_clients.push(client);
        }
        NETWORK_CHANGES.add(change).await;
    }
    pub async fn remove<T>(&self, client: &T) -> bool
    where T: GetId + GetWeak
    {
        let change;
        {
            let mut locked_clients = self.registered_clients.write().await;
            let position = locked_clients
                .iter()
                .position(&|locked_client: &RegisteredClient| locked_client.id == client.get_id());
            if let None = position {
                return false;
            }
            change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(client.weak())).unwrap();
            locked_clients.remove(position.unwrap());
        }
        NETWORK_CHANGES.add(change).await;
        true
    }
    pub async fn get_network_state(&self) -> NetworkStateMessage {
        NetworkStateMessage::new(&self.to_weak().await)
    }

    pub async fn update_clients(&self, updates: String) {
        let action = ServerActions::UpdateClient(updates);
        let mut dead_clients: Vec<RegisteredClient> = vec![];
        {
            let locked_clients = self.registered_clients.read().await;
            for client in locked_clients.iter() {
                // TODO create a better way of removing clients
                if let Err(_) = client.sender.send(action.clone()).await {
                    dead_clients.push(client.clone())
                }
            }
        }
        for client in dead_clients.iter() {
            self.remove(client).await;
        }
    }
}

pub trait GetId {
    fn get_id(&self) -> u64;
}

pub trait GetWeak {
    fn weak(&self) -> WeakClient;
}

pub trait GetAddr {
    fn get_addr(&self) -> &std::net::SocketAddr;
}
pub struct Client {
    pub id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
    stream: Box<tokio::io::BufReader<tokio::net::TcpStream>>,
    sender: Sender<ServerActions>,
    receiver: Box<Receiver<ServerActions>>
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String, stream: BufReader<TcpStream>) -> Client {
        let (tx, rx): (Sender<ServerActions>, Receiver<ServerActions>) = mpsc::channel(32);
        Client {
            id: KNOWN_CLIENTS.next_id(),
            addr,
            name,
            stream: Box::new(stream),
            sender: tx,
            receiver: Box::new(rx),
        }
    }
    pub async fn vibe_check(&mut self) -> bool {
        let mut buf = String::new();
        let response = GenericMessage::new(MessageType::VibeCheck, Status::Ok, None);
        let response = serde_json::to_string(&response).unwrap();
        let result = self.stream.write_all(response.as_bytes()).await;
        if let Err(_) = result {
            return false;
        }
        let result = self.stream.read_line(&mut buf).await;
        if let Err(_) = result {
            return false;
        }
        println!("Got response from vibe-check!: {}", buf);
        let response: Result<GenericMessage, serde_json::Error> = serde_json::from_str(&buf);
        if let Err(_) = response {
            println!("Error while vibe-checking a client: cannot parse client's response");
            return false;
        }
        // TODO this keeps crashing, fix it
        let response = response.unwrap();
        if let MessageType::StillAlive = response.response_type {
            println!("Error while vibe-checking a client: wrong message type");
            return false;
        }
        true
    }
    pub async fn update_client(&mut self, updates: &String) -> std::io::Result<()> {
        if updates.is_empty() {
            return Ok(())
        }
        let mut buf = String::new();
    
    
        self.stream.write_all(updates.as_bytes()).await?;
        let _ = self.stream.read_line(&mut buf).await?;
        let response: Result<GenericMessage, serde_json::Error> = serde_json::from_str(&buf);
        if let Err(_) = response {
            println!("Error while updating client: cannot parse client's response");
            return Ok(())
        }
        let response = response.unwrap();
        
        if matches!(response.status, Status::Error) {
            match response.error {
                Some(e) => println!("Error while updating client: {}", e),
                None => println!("Error while updating client, but no error description was given"),
            }
        }
    
        Ok(())
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
            name: self.name.clone(),
        }
    }
}
impl GetAddr for Client {
    fn get_addr(&self) -> &std::net::SocketAddr {
        &self.addr
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredClient {
    pub id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
    pub sender: Sender<ServerActions>
}

impl RegisteredClient {
    pub fn from(client: &Client) -> RegisteredClient {
        RegisteredClient {
            id: client.id,
            addr: client.addr,
            name: client.name.clone(),
            sender: client.sender.clone()
        }
    }
}

impl GetId for RegisteredClient  {
    fn get_id(&self) -> u64 {
        self.id
    }
}
impl GetWeak for RegisteredClient {
    fn weak(&self) -> WeakClient {
        WeakClient {
            id: self.id,
            addr: self.addr,
            name: self.name.clone(),
        }
    }
}
impl GetAddr for RegisteredClient  {
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
        self.clone()
    }
}

impl GetAddr for WeakClient {
    fn get_addr(&self) -> &std::net::SocketAddr {
        &self.addr
    }
}