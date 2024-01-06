use serde_json::Value;
use tokio::{net::{TcpListener, TcpStream, tcp::WriteHalf}, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}, time};
use lazy_static::lazy_static;
use tokio::time::sleep;
use tokio::sync::{Mutex, mpsc};
use std::{sync::Arc, ops::Index};


mod client;
mod responses;

use crate::{command_args::ParsedArgs, comms::{client::Client, responses::{ RequestType, NetworkChange, get_request_type_str, NetworkChangeType}}};

use self::responses::{Status, GenericMessage, NetworkChanges, DeadClient, NetworkState};

lazy_static! {
    static ref KNOWN_CLIENTS: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    static ref NETWORK_CHANGES: Arc<Mutex<NetworkChanges>> = Arc::new(Mutex::new(NetworkChanges::new(vec![])));
}

pub async fn start_server(args: ParsedArgs) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(addr).await?;

    println!("Listener started at port: {}", args.port);
    
    let mut handles = vec![];

    handles.push(tokio::spawn(serve_client(args, listener)));
    println!("Listening for client requests");
    handles.push(tokio::spawn(check_if_dead(args)));
    println!("Vibe-checker ready. Vibe check every {} secs", args.vibe_check_interval.as_secs());
    handles.push(tokio::spawn(check_updates(args)));
    println!("Client updater ready. Update every {} secs", args.update_client_interval.as_secs());

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

async fn serve_client(args: ParsedArgs, listener: TcpListener) {
    let mut buf = String::new();
    loop {
        buf.clear();
        let (mut stream, addr) = match listener.accept().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while accepting client: {:?}", e);
                continue;
            },
        };
        
        println!("Serving new client: {}", addr);
        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);

        let _bytes_read = match reader.read_line(&mut buf).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while serving client: {:?}", e);
                continue;
            }
        };
        let result = match get_request_type_str(&buf) {
            Ok((RequestType::JoinNetwork, value)) => join_network(&mut writer, addr, value).await,
            Ok((RequestType::FoundDeadClient, value)) => found_dead_client(&mut writer, value).await,
            Ok((RequestType::ExitNetwork, _)) => exit_network(&mut writer, addr).await,
            Ok((RequestType::GetNetworkState, _)) => give_network_state(&mut writer).await,
            Err(e) => {
                println!("Cannot parse client's request: {}", e);
                wrong_request(&mut writer).await
            }
            Ok((_other_request_type, _)) => {
                println!("Wrong request provided by client",);
                wrong_request(&mut writer).await
            },
        };
    
        if let Err(e) = result {
            eprintln!("Error while serving client: {:?}", e);
        }
    }
}

async fn wrong_request(writer: &mut WriteHalf<'_>) -> std::io::Result<()> {
    let error = "Wrong request";
    let response = GenericMessage::result(Status::Error, Some(error));
    let response = serde_json::to_string(&response).unwrap();
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}


async fn give_network_state(writer: &mut WriteHalf<'_>) -> std::io::Result<()> {
    let network_state;
    {
        let locked_clients = KNOWN_CLIENTS.lock().await;
        network_state = NetworkState::new(locked_clients.clone());
    }
    let response = serde_json::to_string(&network_state).unwrap();
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn exit_network(
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

async fn found_dead_client(
    writer: &mut WriteHalf<'_>, 
    request: Value) -> std::io::Result<()> 
{
    let parsed_request: Result<DeadClient, serde_json::Error> = serde_json::from_str(&request.to_string());
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

async fn join_network(
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

async fn check_updates(args: ParsedArgs) {
    loop {
        sleep(args.update_client_interval).await; // Wait between checks
        let locked_clients = KNOWN_CLIENTS.lock().await;
        let locked_changes = NETWORK_CHANGES.lock().await;
        for client in locked_clients.iter() {
            let result = update_client(&locked_changes, client).await;
            if let Err(e) = result {
                println!("Error while updating client {}: {}", client.name, e);
            }
        }
    }
}

async fn update_client(updates: &NetworkChanges, client: &Client) -> std::io::Result<()> {
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

// Function checking for dead clients
async fn check_if_dead(args: ParsedArgs) {
    loop {
        sleep(args.vibe_check_interval).await; // Wait between checks
        println!("Runs vibe check!");

        // Vector containing ids of dead clients
        let dead_client_ids: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
        // Vector containing handles of all tokio threads created in this function
        let mut handles = vec![];
        
        // We want to drop all senders after this block,
        // so we wrap it in it's scope
        {
            // Channel to communicate between vibe checkers and a thread
            // that pushes dead client ids into "dead_client_ids" vector
            // It is like this to not block the dead_client_ids's mutex
            let (tx, mut rx) = mpsc::channel(64);

            // We copy clients to avoid locking data for to long for other tasks
            let copied_clients;
            {
                copied_clients = KNOWN_CLIENTS.lock().await.clone();
            }

            // Thread appending dead_client_id vector with ids
            let dead_client_ids_arc = Arc::clone(&dead_client_ids);
            handles.push(tokio::spawn(async move {
                let mut results = dead_client_ids_arc.lock().await;
                loop {
                    let id = rx.recv().await;
                    match id {
                        Some(id) => results.push(id),
                        None => return,
                    }
                }
            }));

            // Vibe checkers. If client is dead they broadcast it
            // thru the channel
            for client in copied_clients.iter() {
                let copy_tx = tx.clone();
                let client = client.clone();
                handles.push(tokio::spawn(async move {
                    tokio::select! {
                        if_alive = vibe_check(&client) => {
                            if !if_alive {
                                println!("Client {} is dead", client.name);
                                copy_tx.send(client.id).await.unwrap();
                            }
                        }
                        _ = sleep(time::Duration::from_secs(10)) => {
                            println!("Client {} is dead", client.name);
                            copy_tx.send(client.id).await.unwrap();
                        }
                    }
                }));
            }
        }
        // Await for all threads to complete
        for handle in handles {
            let result = handle.await;
            if let Err(e) = result {
                println!("Handle error during vibe check: {:?}", e);
            }
        }
        // Remove dead clients from the list
        let mut changes = vec![];
        {
            let locked_dead_clients_ids = dead_client_ids.lock().await;
            let mut locked_clients = KNOWN_CLIENTS.lock().await;
            for client in locked_clients.iter() {
                if locked_dead_clients_ids.contains(&client.id) {
                    let change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(client)).unwrap();
                    changes.push(change);
                }
            }
            locked_clients.retain(|client| !locked_dead_clients_ids.contains(&client.id));
        }

        append_changes(changes).await;

        println!("End of vibe check!");
    }
}

// Checks if client is dead
async fn vibe_check(client: &client::Client) -> bool {
    let mut buf = String::new();
    let stream = TcpStream::connect(client.addr).await;
    if let Err(_) = stream {
        return false;
    }
    let mut stream = stream.unwrap(); // Can safely unwrap

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let response = GenericMessage::new(RequestType::VibeCheck, Status::Ok, None);
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
    if let RequestType::StillAlive = response.response_type {
        return false;
    }
    true
}

async fn append_changes(mut changes: Vec<NetworkChange>) {
    let mut locked_changes = NETWORK_CHANGES.lock().await;
    locked_changes.changes.append(&mut changes);
}

async fn remove_client_from_clients(client: &Client) -> Option<()>{
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