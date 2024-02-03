use tokio::sync::mpsc::Receiver;

use tokio::{io::{AsyncBufReadExt, BufReader}, net::TcpStream};

use crate::comms::client::client_reqs;
use crate::comms::network::api::{get_request_type_str, MessageType};
use crate::comms::wrong_request;

use super::{Client, ServerActions};

pub async fn client_handler(mut client: Client) {
    //let parsed_request: Result<DeadClientMessage, serde_json::Error> = serde_json::from_str(&request.to_string());
    let mut buffer = String::new();
    
    loop {
        tokio::select! {
            val = next_request(&mut client.stream, &mut buffer) => {
                if let Err(e) = val {
                    eprintln!("Error while serving a registered client: {}", e);
                    continue
                }
                let result = match get_request_type_str(&buffer) {
                    Ok((MessageType::GetNetworkState, _)) => client_reqs::give_network_state(&mut client).await,
                    Ok((MessageType::ExitNetwork, _)) => client_reqs::exit_network(&mut client).await,
                    Ok((MessageType::FoundDeadClient, value)) => client_reqs::found_dead_client(&mut client, value).await,
                    Err(e) => {
                        println!("Cannot parse client's request: {}", e);
                        let result = wrong_request(&mut client.stream).await;
                        if let Err(err) = result {
                            eprintln!("Error while serving a registered client: {}", err);
                        }  
                        continue;
                    }
                    Ok((_other_request_type, _)) => {
                        println!("Wrong request provided by client",);
                        let result = wrong_request(&mut client.stream).await;
                        if let Err(err) = result {
                            eprintln!("Error while serving a registered client: {}", err);
                        }
                        continue;
                    },
                };

                if let Err(err) = result {
                    eprintln!("Error while serving a registered client: {}", err);
                }
            }
            _val = server_request(&mut client.receiver) => {
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