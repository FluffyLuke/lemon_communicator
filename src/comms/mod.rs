use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}};
use tokio::time::sleep;

use crate::{command_args::ParsedArgs, comms::{client::{client_handler, client_reqs::join_network}, network::api::{get_request_type_str, MessageType}}};

use self::{network::{api::{GenericMessage, Status}, NETWORK_CHANGES}, client::KNOWN_CLIENTS};

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

async fn serve_client(_args: ParsedArgs, listener: TcpListener) {
    let mut buf = String::new();
    let mut handles = vec![];
    loop {
        buf.clear();
        let (stream, addr) = match listener.accept().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while accepting client: {:?}", e);
                continue;
            },
        };
        
        println!("Serving new client: {}", addr);

        let mut buf_stream = BufReader::new(stream);

        let _bytes_read = match buf_stream.read_line(&mut buf).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while serving client: {:?}", e);
                continue;
            }
        };

        let result = match get_request_type_str(&buf) {
            Ok((MessageType::JoinNetwork, value)) => join_network(buf_stream, addr, value).await,
            Err(e) => {
                println!("Cannot parse client's request: {}", e);
                let result = wrong_request(&mut buf_stream).await;
                if let Err(e) = result {
                    eprintln!("Error while serving client: {:?}", e);
                }
                continue;
            }
            Ok((_other_request_type, _)) => {
                println!("Wrong request provided by client",);
                let result = wrong_request(&mut buf_stream).await;
                if let Err(e) = result {
                    eprintln!("Error while serving client: {:?}", e);
                }
                continue;
            },
        };
    
        let handle = match result {
            Ok(client) => tokio::spawn(client_handler::client_handler(client)),
            Err(err) => {
                eprintln!("Error while serving client: {:?}", err);
                continue;
            },
        };
        handles.push(handle);
    }
}

async fn wrong_request(socket: &mut BufReader<TcpStream>,) -> std::io::Result<()> {
    let error = "Wrong request";
    let response = GenericMessage::result(Status::Error, Some(error));
    let response = serde_json::to_string(&response).unwrap();
    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn check_updates(args: ParsedArgs) {
    loop {
        sleep(args.update_client_interval).await; // Wait between checks
        println!("Updating clients!");

        let changes = NETWORK_CHANGES.get_changes_json().await;
        if let None = changes {
            println!("No changes found!");
            continue;
        }
        let changes = changes.unwrap();
        KNOWN_CLIENTS.update_clients(changes).await;
        NETWORK_CHANGES.reset_changes().await;
        println!("End of update!");
    }
}



async fn check_if_dead(args: ParsedArgs) {
    loop {
        sleep(args.vibe_check_interval).await; // Wait between checks
        println!("Runs vibe check!");
        KNOWN_CLIENTS.vibe_check().await;
        println!("End of vibe check")
    }
}