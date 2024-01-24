use serde_json::Value;
use tokio::{net::tcp::WriteHalf, io::{AsyncWriteExt, BufReader}};

use crate::comms::{network::{api::{GenericMessage, Status, NetworkChange, NetworkChangeType, DeadClientMessage, NetworkStateMessage}, vibe_check}, client::{Client, KNOWN_CLIENTS}};

use super::GetAddr;


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

// TODO FIX THIS 
// pub async fn found_dead_client(client: &mut Client, request: Value) -> std::io::Result<()> {
//     let parsed_request: Result<DeadClientMessage, serde_json::Error> = serde_json::from_str(&request.to_string());
//     if let Err(_) = parsed_request {
//         let error = "Wrong request";
//         let response = GenericMessage::result(Status::Error, Some(error));
//         let response = serde_json::to_string(&response).unwrap();
//         client.stream.write_all(response.as_bytes()).await?;
//     }

//     let unwrapped_request = parsed_request.unwrap();
//     let result = vibe_check(&unwrapped_request.client).await;
//     let
//     if !result {
//         let _result = KNOWN_CLIENTS.remove(&unwrapped_request.client).await;
//     }
//     let response = GenericMessage::result(Status::Ok, None);
//     let response = serde_json::to_string(&response).unwrap();
//     client.stream.write_all(response.as_bytes()).await?;
//     Ok(())
// }

// TODO Change the way value is being unpacked, maybe into a struct 
pub async fn join_network(
    mut socket: tokio::net::TcpStream,
    addr: std::net::SocketAddr, 
    request: Value) 
    -> std::io::Result<()> {
    let client_name = request.get("client")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str());

    if let None = client_name {
        let error = "Client's name not found";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        socket.write_all(response.as_bytes()).await?;
        return Ok(())
    }

    let mut new_client = Client::new(addr,client_name.unwrap().to_string(), BufReader::new(socket));
    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    new_client.stream.write_all(response.as_bytes()).await?;
    KNOWN_CLIENTS.add(new_client);
    Ok(())
}