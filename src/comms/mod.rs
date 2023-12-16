use tokio::{net::{TcpListener, TcpStream, tcp::{WriteHalf, ReadHalf}}, io::{AsyncWriteExt, AsyncReadExt, BufReader, AsyncBufReadExt}, time};
use lazy_static::lazy_static;
use tokio::time::sleep;
use tokio::sync::{Mutex, mpsc};
use std::{str::from_utf8, sync::Arc};

mod client;

use crate::{command_args::ParsedCommands, comms::client::Client};

const CLIENT_INFO_SIZE: usize = 255;

// Client requests
const JOIN_NETWORK: u8 = 0x1;
const STILL_ALIVE: u8 = 0x2;

// Server requests
const ACCEPT_CLIENT: u8 = 0x10;
const VIBE_CHECK: u8 = 0x11;
const UPDATE_CLIENT: u8 = 0x12;

// Error messages
const UNKNOWN_REQUEST: u8 = 0xF1;
const WRONG_CLIENT_INFO: u8 = 0xF2;

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
    //let mut buf = String::new();
    loop {
        //buf.clear();
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
        let request = match reader.read_u8().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while serving client: {:?}", e);
                continue;
            }
        };
        reader.consume(4);
        // let request: u8 = match buf.parse::<u8>() {
        //     Ok(request) => request,
        //     Err(e) => {
        //         eprintln!("Wrong client request: {e}");
        //         continue;
        //     }
        // };
        
        let result = match request {
            JOIN_NETWORK => join_network(&mut reader, &mut writer, addr).await,
            48 => join_network(&mut reader, &mut writer, addr).await,
            _ => {
                println!("Wrong request {}", request);
                let _result = writer.write(&[UNKNOWN_REQUEST]).await;
                continue;
            }
        };
    
        if let Err(e) = result {
            eprintln!("Error while serving client: {:?}", e);
        }
    }
}

async fn join_network(reader: &mut BufReader<ReadHalf<'_>>, writer: &mut WriteHalf<'_>, addr: std::net::SocketAddr) -> std::io::Result<()> {
    writer.write_all(&[ACCEPT_CLIENT]).await?;
    let mut buf = [0; CLIENT_INFO_SIZE];
    let bytes_read = reader.read(&mut buf).await?;

    if bytes_read == 0 {
        return Err(std::io::ErrorKind::ConnectionAborted.into());
    }

    let name = match from_utf8(&buf) {
        Ok(name) => name.to_string(),
        Err(_) => {
            writer.write_all(&[WRONG_CLIENT_INFO]).await?;
            return Ok(());
        }
    };

    println!("New client was registered: {:?}", buf);

    let new_client = Client::new(addr, name);
    KNOWN_CLIENTS.lock().await.push(new_client);
    Ok(())
}

async fn update_clients() {
    let time_to_wait = time::Duration::from_secs(10);
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
    let time_to_wait = time::Duration::from_secs(10); // Time to wait between each check
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
    let stream = TcpStream::connect(client.addr).await;
    if let Err(_) = stream {
        return false;
    }
    let mut stream = stream.unwrap();
    let result = stream.write_all(&[VIBE_CHECK]).await;
    if let Err(_) = result {
        return false;
    }
    let client_message = stream.read_u8().await;
    if let Err(_) = client_message {
        return false;
    }
    if client_message.unwrap() != STILL_ALIVE {
        return false;
    }
    true
}