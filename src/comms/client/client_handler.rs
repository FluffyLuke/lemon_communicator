use tokio::{io::{AsyncBufReadExt, BufReader}, net::TcpStream};

use crate::comms::client::client_reqs;
use crate::comms::network::api::{get_request_type_str, MessageType};
use crate::comms::wrong_request;

use super::{Client, ServerActions};

pub async fn client_handler(mut client: Client) {
    let mut buffer = String::new();
    loop {
        //client.stream.write_all(serde_json::to_string(&message).unwrap().as_bytes()).await.unwrap();
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
                        println!("Cannot parse registered client's request: {}", e);
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
            val = client.receiver.recv() => {
                let if_dead = server_request(&mut client, val.unwrap()).await;
                // If server sends ServerActions::Disconnect, then break the loop and stop the task
                if if_dead {
                    break
                }
            }
        }
        buffer.clear();
    }
}

async fn next_request(stream: &mut BufReader<TcpStream>, buffer: &mut String) -> std::io::Result<()>{
    stream.read_line(buffer).await?;
    Ok(())
}

// TODO if fails, make it check if client is alive
async fn server_request(client: &mut Client, request: ServerActions) -> bool{
    match request {
        ServerActions::UpdateClient(sender, updates) => {
            let result = client.update_client(&updates).await;
            if let Err(err) = result {
                eprintln!("Error while updating a registered client: {}", err);
                sender.send(false).unwrap();
                return true
            }
            sender.send(true).unwrap();
            true
        }
        ServerActions::CheckIfDead(sender) => {
            let if_dead = client.vibe_check().await;
            sender.send(!if_dead).unwrap();
            true
        }
        ServerActions::Disconnect => false
    }
}