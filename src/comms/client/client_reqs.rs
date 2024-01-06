use serde_json::Value;
use tokio::{net::tcp::WriteHalf, io::AsyncWriteExt};

use crate::comms::{network::{api::{GenericMessage, Status, NetworkChange, NetworkChangeType, DeadClientMessage, NetworkStateMessage}, append_changes, vibe_check}, client::{Client, KNOWN_CLIENTS}};

use super::{remove_client_from_clients, remove_client_from_clients_by_addr};

pub async fn give_network_state(writer: &mut WriteHalf<'_>) -> std::io::Result<()> {
    let network_state;
    {
        let locked_clients = KNOWN_CLIENTS.lock().await;
        network_state = NetworkStateMessage::new(locked_clients.clone());
    }
    let response = serde_json::to_string(&network_state).unwrap();
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn exit_network(
    writer: &mut WriteHalf<'_>,
    addr: std::net::SocketAddr) -> std::io::Result<()>
{
    let result = remove_client_from_clients_by_addr(addr).await;
    if result == None {
        let error = "Client not found";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        writer.write_all(response.as_bytes()).await?;
    }

    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn found_dead_client(
    writer: &mut WriteHalf<'_>, 
    request: Value) -> std::io::Result<()> 
{
    let parsed_request: Result<DeadClientMessage, serde_json::Error> = serde_json::from_str(&request.to_string());
    if let Err(_) = parsed_request {
        let error = "Wrong request";
        let response = GenericMessage::result(Status::Error, Some(error));
        let response = serde_json::to_string(&response).unwrap();
        writer.write_all(response.as_bytes()).await?;
    }

    let unwrapped_request = parsed_request.unwrap();
    let result = vibe_check(&unwrapped_request.client).await;
    if !result {
        let _result = remove_client_from_clients(&unwrapped_request.client).await;
    }
    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn join_network(
    //reader: &mut BufReader<ReadHalf<'_>>, 
    writer: &mut WriteHalf<'_>, 
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
        writer.write_all(response.as_bytes()).await?;
        return Ok(())
    }

    let new_client = Client::new(addr, client_name.unwrap().to_string());

    println!("New client registered! {:?}", new_client);
    let change = NetworkChange::new(NetworkChangeType::JoinNetwork, Some(&new_client)).unwrap();
    append_changes(vec![change]).await;
    {
        KNOWN_CLIENTS.lock().await.push(new_client);
    }
    let response = GenericMessage::result(Status::Ok, None);
    let response = serde_json::to_string(&response).unwrap();
    writer.write_all(response.as_bytes()).await?;

    Ok(())
}