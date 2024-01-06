use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::{sync::Mutex, net::TcpStream, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}};

use self::api::{NetworkChange, GenericMessage, Status, NetworkChangesMessage, MessageType};

use super::client::Client;

pub mod api;

lazy_static! {
    pub static ref NETWORK_CHANGES: Arc<Mutex<NetworkChangesMessage>> = Arc::new(Mutex::new(NetworkChangesMessage::new(vec![])));
}

pub async fn append_changes(mut changes: Vec<NetworkChange>) {
    let mut locked_changes = NETWORK_CHANGES.lock().await;
    locked_changes.changes.append(&mut changes);
}

pub async fn update_client(updates: &NetworkChangesMessage, client: &Client) -> std::io::Result<()> {
    if updates.changes.is_empty() {
        return Ok(())
    }
    let mut buf = String::new();
    let message = serde_json::to_string(updates).unwrap();

    let mut stream = TcpStream::connect(client.addr).await?;
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    writer.write_all(message.as_bytes()).await?;
    let _ = reader.read_line(&mut buf).await?;
    let response: Result<GenericMessage, serde_json::Error> = serde_json::from_str(&buf);
    if let Err(_) = response {
        println!("Updating client failed. Cannot parse client's response");
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
pub async fn vibe_check(client: &Client) -> bool {
    let mut buf = String::new();
    let stream = TcpStream::connect(client.addr).await;
    if let Err(_) = stream {
        return false;
    }
    let mut stream = stream.unwrap(); // Can safely unwrap

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let response = GenericMessage::new(MessageType::VibeCheck, Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    let result = writer.write_all(response.as_bytes()).await;
    if let Err(_) = result {
        return false;
    }
    let result = reader.read_line(&mut buf).await;
    if let Err(_) = result {
        return false;
    }
    let response: Result<GenericMessage, serde_json::Error> = serde_json::from_str(&buf);
    if let Err(_) = response {
        println!("Vibe-checking. Cannot parse client's response");
    }
    let response = response.unwrap();
    if let MessageType::StillAlive = response.response_type {
        return false;
    }
    true
}