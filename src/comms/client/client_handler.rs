use tokio::sync::mpsc::Receiver;

use tokio::{io::{AsyncBufReadExt, BufReader}, net::TcpStream};

use crate::comms::client::client_reqs::{give_network_state, join_network};
use crate::comms::network::api::{get_request_type_str, MessageType};
use crate::comms::wrong_request;

use super::{Client, ServerActions};

pub async fn client_handler(mut client: Client) {
    //let parsed_request: Result<DeadClientMessage, serde_json::Error> = serde_json::from_str(&request.to_string());
    let mut buffer = String::new();
    let mut stream = &client.stream;
    let receiver = &client.receiver;
    loop {
        tokio::select! {
            val = next_request(&mut *stream, &mut buffer) => {
                if let Err(e) = val {
                    eprintln!("Error while serving a registered client: {}", e);
                    continue
                }
                let result = match get_request_type_str(&buffer) {
                    Ok((MessageType::GetNetworkState, _)) => give_network_state(&mut client),
                    Err(e) => {
                        println!("Cannot parse client's request: {}", e);
                        let result = wrong_request(&mut stream).await;
                        if let Err(e) = result {
                            eprintln!("Error while serving a registered client: {}", e);
                        }
                        continue;
                    }
                    Ok((_other_request_type, _)) => {
                        println!("Wrong request provided by client",);
                        let result = wrong_request(&mut stream).await;
                        if let Err(e) = result {
                            eprintln!("Error while serving a registered client: {}", e);
                        }
                        continue;
                    },
                };
            }
            val = server_request(&mut *receiver) => {
                // TODO
            }
        }
    }
}

async fn next_request(stream: &mut BufReader<TcpStream>, buffer: &mut String) -> std::io::Result<()>{
    stream.read_line(buffer).await?;
    Ok(())
}
async fn server_request(receiver: &mut Receiver<ServerActions>) {
    todo!()
}