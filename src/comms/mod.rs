use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt, AsyncReadExt, BufReader}};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use std::{str, sync::Arc};

mod client;

use crate::{command_args::ParsedCommands, comms::client::Client};

const CLIENT_INFO_SIZE: usize = 255;

const JOIN_NETWORK: u8 = 0x1;
const ACCEPT_CLIENT: u8 = 0x2;

const UNKNOWN_REQUEST: u8 = 0x10;
const WRONG_CLIENT_INFO: u8 = 0x1A;

lazy_static! {
    static ref KNOWN_CLIENTS: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
}

pub async fn start_server(commands: ParsedCommands) -> std::io::Result<()>{
    let addr = format!("127.0.0.1:{}", commands.port);
    let listener = TcpListener::bind(addr).await?;

    let handle1 = tokio::spawn(check_if_dead());
    let handle2 = tokio::spawn(update_clients());
    let handle3 = tokio::spawn(serve_client(listener));

    handle1.await.unwrap();
    handle2.await.unwrap();
    handle3.await.unwrap();

    Ok(())
}

async fn serve_client(listener: TcpListener) {
    loop {
        let (mut sock, addr) = match listener.accept().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while accepting new client: {:?}", e);
                continue;
            },
        };
        let (reader, _writer) = sock.split();
        let mut reader = BufReader::new(reader);
        let request = match reader.read_u8().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while serving client: {:?}", e);
                continue;
            }
        };

        let result = match request {
            JOIN_NETWORK => join_network(&mut sock, addr).await,
            48 => join_network(&mut sock, addr).await, //remove later
            49 => {
                for c in KNOWN_CLIENTS.lock().await.iter() {
                    println!("{:?}", c);
                }
                Ok(())
            }
            _ => {
                println!("error {}", request);
                let _result = sock.write(&[UNKNOWN_REQUEST]).await;
                continue;
            }
        };

        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    }
}

async fn join_network(socket: &mut TcpStream, addr: std::net::SocketAddr) -> std::io::Result<()> {
    println!("e");
    let mut buf: [u8; CLIENT_INFO_SIZE] = [0; CLIENT_INFO_SIZE];
    let bytes_read: usize;
    socket.write_all(&[ACCEPT_CLIENT]).await?;

    let result = socket.read(&mut buf).await;

    bytes_read = match result {
        Ok(bytes_n) => bytes_n,
        Err(e) => return Err(e),
    };

    if bytes_read == 0 {
        return Err(std::io::ErrorKind::ConnectionAborted.into());
    }

    let parsed_buf = str::from_utf8(&buf);
    if let Err(_) = parsed_buf {
        socket.write_all(&[WRONG_CLIENT_INFO]).await?;
        return Ok(());
    }
    let name = parsed_buf.unwrap();

    // TODO : Fix this later
    let name = name.replace("\0", "").trim().to_string();

    println!("New client was registered: {:?}", name);

    let new_client = Client::new(addr, name);
    KNOWN_CLIENTS.lock().await.push(new_client);
    Ok(())
}

async fn update_clients() {
    println!("TODO");
}

async fn check_if_dead() {
    println!("TODO2");
}