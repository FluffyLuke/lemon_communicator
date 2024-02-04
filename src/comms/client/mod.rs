use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};


use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time;
use crate::comms::network::api::{GenericMessage, MessageType, Status};

use super::network::NETWORK_CHANGES;
use super::network::api::{NetworkChange, NetworkChangeType, NetworkStateMessage};

pub mod client_reqs;
pub mod client_handler;

lazy_static!{
    pub static ref KNOWN_CLIENTS: KnownClients = KnownClients::create();
}

#[derive(Debug)]
pub enum ServerActions {
    UpdateClient(oneshot::Sender<bool>, String),
    CheckIfDead(oneshot::Sender<bool>),
    Disconnect
}

#[derive(Debug)]
pub enum CheckIfDeadError {
    NotFound,
    IOError(std::io::Error),
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
            let found_client = locked_clients.iter().find(|x: &&RegisteredClient| x.id == client.get_id()).unwrap();
            found_client.sender.send(ServerActions::Disconnect).await.unwrap();
            locked_clients.remove(position.unwrap());
        }
        NETWORK_CHANGES.add(change).await;
        true
    }
    pub async fn get_network_state(&self) -> NetworkStateMessage {
        NetworkStateMessage::new(&self.to_weak().await)
    }

    pub async fn remove_multiple<T>(&self, dead_clients: &Vec<T>) -> bool
    where T: GetId + GetWeak
    {
        let mut changes: Vec<NetworkChange> = vec![];
        for client in dead_clients.iter() {
            let change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(client.weak())).unwrap();
            changes.push(change);
        }

        {
            let mut locked_clients = self.registered_clients.write().await;
            for client in locked_clients.iter() {
                if dead_clients.iter().find(|&x| x.get_id() == client.id).is_some() {
                    client.sender.send(ServerActions::Disconnect).await.unwrap();
                }
            }
            locked_clients.retain(|x| dead_clients.iter().find(|&y| y.get_id() == x.id).is_some());
        }
        NETWORK_CHANGES.add_multiple(changes).await;
        true
    }

    pub async fn update_clients(&self, updates: String) {
        // TODO change this tuple, so it will contain only ids, not whole clients.
        // This will require changes in remove method
        let mut receivers: Vec<(oneshot::Receiver<bool>, WeakClient)> = vec![]; 
        {
            let locked_clients = self.registered_clients.read().await;
            for client in locked_clients.iter() {
                let (tx, rx): (oneshot::Sender<bool>, oneshot::Receiver<bool>) = oneshot::channel();
                // TODO do something about cloning the string possibly hundreds of times
                client.sender.send(ServerActions::UpdateClient(tx, updates.clone())).await.unwrap();
                receivers.push((rx, client.weak()));
            }
        }
        for (rx, client) in receivers {
            if !rx.await.unwrap() {
                self.remove(&client).await;
            }
        }
    }
    pub async fn check_if_exists<T>(&self, client: &T) -> bool
    where T: GetId {
        let id = client.get_id();
        let locked_clients = self.registered_clients.read().await;
        locked_clients.iter().any(|x| x.id == id)
    }
    // TODO make better error message is oneshot fails
    pub async fn check_if_dead<T>(&self, client: &T) -> Result<bool, CheckIfDeadError>
    where T: GetId {
        let sender_copy;
        {
            let id = client.get_id();
            let locked_clients = self.registered_clients.read().await;
            let found_client = locked_clients.iter().find(|&x| x.id == id);
            if found_client.is_none() {
                return Err(CheckIfDeadError::NotFound);
            }
            sender_copy = found_client.unwrap().sender.clone();
        }
        let (tx, rx): (oneshot::Sender<bool>, oneshot::Receiver<bool>) = oneshot::channel();
        sender_copy.send(ServerActions::CheckIfDead(tx)).await.unwrap();
        let if_dead = rx.await.unwrap();
        Ok(if_dead)
    }
    // Like check_if_dead, but checks all clients
    pub async fn vibe_check(&self) {
        let mut sender_copies = vec![];
        let mut dead_clients = vec![];
        {
            let locked_clients = self.registered_clients.read().await;

            for client in locked_clients.iter() {
                let (tx, rx): (oneshot::Sender<bool>, oneshot::Receiver<bool>) = oneshot::channel();
                client.sender.send(ServerActions::CheckIfDead(tx)).await.unwrap();
                sender_copies.push((rx, client.weak()));       
            };
        }
        for (receiver, weak_client) in sender_copies {
            if receiver.await.unwrap() {
                dead_clients.push(weak_client)
            }
        }
        self.remove_multiple(&dead_clients).await;
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
    stream: tokio::io::BufReader<tokio::net::TcpStream>,
    sender: mpsc::Sender<ServerActions>,
    receiver: mpsc::Receiver<ServerActions>
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String, stream: BufReader<TcpStream>) -> Client {
        let (tx, rx): (mpsc::Sender<ServerActions>, mpsc::Receiver<ServerActions>) = mpsc::channel(32);
        Client {
            id: KNOWN_CLIENTS.next_id(),
            addr,
            name,
            stream: stream,
            sender: tx,
            receiver: rx,
        }
    }
    // TODO make it wait n secs before aborting
    pub async fn vibe_check(&mut self) -> bool {
        tokio::select! {
            if_alive = self.check_if_dead() => if_alive,
            _ = time::sleep(time::Duration::from_secs(3)) => false
        }
    }
    async fn check_if_dead(&mut self) -> bool{
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

impl GetId for u64 {
    fn get_id(&self) -> u64 {
        *self
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
    pub sender: mpsc::Sender<ServerActions>
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