use std::fmt;

use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::comms::network::api::{DeadClientMessage, GenericMessage, Status};

use super::{Client, RegisteredClient, KNOWN_CLIENTS};


pub async fn give_network_state(client: &mut Client) -> std::io::Result<()> {
    let network_state = KNOWN_CLIENTS.get_network_state().await;
    let response = serde_json::to_string(&network_state).unwrap();
    client.stream.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn exit_network(client: &mut Client) -> std::io::Result<()> {
    let result = KNOWN_CLIENTS.remove(client).await;
    if !result {
        let error = "Client not found";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        client.stream.write_all(response.as_bytes()).await?;
    }

    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();

    client.stream.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn found_dead_client(client: &mut Client, request: Value) -> std::io::Result<()> {
    let parsed_request: Result<DeadClientMessage, serde_json::Error> = serde_json::from_str(&request.to_string());
    if let Err(_) = parsed_request {
        let error = "Wrong request";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        client.stream.write_all(response.as_bytes()).await?;
    }

    let unwrapped_request = parsed_request.unwrap();
    let result = KNOWN_CLIENTS.check_if_dead(&unwrapped_request.client).await;
    if let Err(_err) = result {
        let _result = KNOWN_CLIENTS.remove(&unwrapped_request.client).await;
    }
    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    client.stream.write_all(response.as_bytes()).await?;
    Ok(())
}

// TODO Change the way value is being unpacked, maybe into a struct 
pub async fn join_network(
    mut socket: tokio::io::BufReader<tokio::net::TcpStream>,
    addr: std::net::SocketAddr, 
    request: Value) 
    -> Result<Client, JoinError> {
    let client_name = request.get("client")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str());

    if let None = client_name {
        let error = "Client's name not found";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        socket.write_all(response.as_bytes()).await.map_err(|err| JoinError::IOError(err))?;
        return Err(JoinError::BadJSON)
    }

    let mut new_client = Client::new(addr,client_name.unwrap().to_string(), socket);
    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    let registered_client = RegisteredClient::from(&new_client);
    new_client.stream.write_all(response.as_bytes()).await.map_err(|err| JoinError::IOError(err))?;
    KNOWN_CLIENTS.add(registered_client).await;
    Ok(new_client)
}

#[derive(Debug)]
pub enum JoinError {
    BadJSON,
    IOError(std::io::Error)
}

impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JoinError::BadJSON=> {
                write!(f, "Request wasn't a JSON or has a bad structure")
            }
            JoinError::IOError(field) => {
                write!(f, "{}", field)
            }
        }
    }
}