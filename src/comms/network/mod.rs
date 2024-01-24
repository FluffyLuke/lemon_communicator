use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::{sync::Mutex, net::TcpStream, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}};

use self::api::{NetworkChange, GenericMessage, Status, NetworkChangesMessage, MessageType};

use super::client::Client;

pub mod api;

lazy_static! {
    pub static ref NETWORK_CHANGES: NetworkChanges = NetworkChanges::new();
}

struct NetworkChanges {
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
    pub async fn get_changes_json(&self) -> String {
        let locked_changes = self.changes.lock().await;
        if locked_changes.changes.is_empty() {
            return String::new()
        }
        serde_json::to_string(&*locked_changes).unwrap()
    }
    pub async fn reset_changes(&self) {
        self.changes.lock().await.changes.clear();
    }
}

pub async fn update_client(updates: String, client: &mut Client) -> std::io::Result<()> {
    if updates.is_empty() {
        return Ok(())
    }
    let mut buf = String::new();


    client.stream.write_all(updates.as_bytes()).await?;
    let _ = client.stream.read_line(&mut buf).await?;
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

// Checks if client is dead
pub async fn vibe_check(client: &mut Client) -> bool {
    let mut buf = String::new();
    let response = GenericMessage::new(MessageType::VibeCheck, Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    let result = client.stream.write_all(response.as_bytes()).await;
    if let Err(_) = result {
        return false;
    }
    let result = client.stream.read_line(&mut buf).await;
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