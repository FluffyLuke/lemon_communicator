use serde::de::value;
use serde_json::Value;
use tokio::{net::{TcpListener, TcpStream, tcp::{WriteHalf, ReadHalf}}, io::{AsyncWriteExt, AsyncReadExt, BufReader, AsyncBufReadExt}, time};
use lazy_static::lazy_static;
use tokio::time::sleep;
use tokio::sync::{Mutex, mpsc};
use std::{str::from_utf8, sync::Arc};

mod client;
mod responses;

use crate::{command_args::ParsedCommands, comms::{client::Client, responses::{ RequestType, get_request_type_str}}};

use self::responses::{result_response, Status, generic_response};

lazy_static! {
    static ref KNOWN_CLIENTS: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
}

pub async fn start_server(commands: ParsedCommands) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", commands.port);
    let listener = TcpListener::bind(addr).await?;
    
    let mut handles = vec![];

    handles.push(tokio::spawn(serve_client(listener)));
    handles.push(tokio::spawn(check_if_dead()));
    handles.push(tokio::spawn(update_clients()));

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

async fn serve_client(listener: TcpListener) {
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
            Ok((RequestType::join_network, value)) => join_network(&mut writer, addr, value).await,
            Err(e) => {
                println!("Wrong request: {}", e);
                //let _result = writer.write(&[UNKNOWN_REQUEST]).await;
                continue;
            }
            _ => todo!(),
        };
    
        if let Err(e) = result {
            eprintln!("Error while serving client: {:?}", e);
        }
    }
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
        let response = result_response(Status::error, Some(error));
        writer.write_all(response.as_bytes()).await?;
        return Ok(())
    }

    let new_client = Client::new(addr, client_name.unwrap().to_string());

    println!("New client registered! {:?}", new_client);
    KNOWN_CLIENTS.lock().await.push(new_client);
    let response = result_response(Status::ok, None);
    writer.write_all(response.as_bytes()).await?;

    Ok(())
}

async fn update_clients() {
    let time_to_wait = time::Duration::from_secs(1200);
    loop {
        sleep(time_to_wait).await; // Wait between checks
        println!("Updating clients!");
        let copied_clients;
        {
            copied_clients = KNOWN_CLIENTS.lock().await.clone();
        }
        let serlialized_clients = serde_json::to_string(&copied_clients).unwrap();
        for client in copied_clients {
            let result = TcpStream::connect(client.addr).await;
            if let Err(_) = result {
                continue;
            }
            let mut stream = result.unwrap();

            let _ = stream.write(serlialized_clients.as_bytes()).await;
        }
    }
}

// Function checking for dead clients
async fn check_if_dead() {
    let time_to_wait = time::Duration::from_secs(1200); // Time to wait between each check
    loop {
        sleep(time_to_wait).await; // Wait between checks
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
                        _ = sleep(time_to_wait) => {
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
        {
            let locked_dead_clients_ids = dead_client_ids.lock().await;
            let mut locked_clients = KNOWN_CLIENTS.lock().await;
            locked_clients.retain(|client| !locked_dead_clients_ids.contains(&client.id));
        }
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
    let mut stream = stream.unwrap();
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let response = generic_response(RequestType::vibe_check, Status::ok, None);

    let result = writer.write_all(response.as_bytes()).await;
    if let Err(_) = result {
        return false;
    }
    let result = reader.read_line(&mut buf).await;
    if let Err(_) = result {
        return false;
    }
    let request_type = match get_request_type_str(&buf) {
        Ok((req_type, _)) => req_type,
        Err(_) => return false,
    };
    if let RequestType::still_alive = request_type {
        return false;
    }
    true
}