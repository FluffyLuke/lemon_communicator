use tokio::{net::{TcpListener, tcp::WriteHalf}, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}, time};
use tokio::time::sleep;
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;

use crate::{command_args::ParsedArgs, comms::{network::{api::{MessageType, NetworkChangeType, NetworkChange, get_request_type_str}, vibe_check, append_changes}, client::client_reqs::{join_network, found_dead_client, exit_network, give_network_state}}};

use self::{network::{api::{GenericMessage, Status}, NETWORK_CHANGES, update_client}, client::KNOWN_CLIENTS};


mod client;
mod network;


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
            Ok((MessageType::JoinNetwork, value)) => join_network(&mut writer, addr, value).await,
            Ok((MessageType::FoundDeadClient, value)) => found_dead_client(&mut writer, value).await,
            Ok((MessageType::ExitNetwork, _)) => exit_network(&mut writer, addr).await,
            Ok((MessageType::GetNetworkState, _)) => give_network_state(&mut writer).await,
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