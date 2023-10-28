use serde_json::to_string;
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt, BufWriter, BufReader}};
use tokio::time::sleep;
use std::{str, string, sync::Arc, time::Duration};

use crate::peer::get_peers;

pub const KNOWN_ADDRESSES_REQUEST: u8 = 1;
//pub const PEER_INFO_REQUEST: u8 = 1;
pub async fn send_peer_data(socket: Arc<TcpListener>) -> std::io::Result<()> {
    let mut request_buf  = [0; 1];
    loop {
        let (mut stream, addr) = socket.accept().await?;
        println!("Got connection from {} asking for peer addresses", addr.to_string());
        let mut reader = BufReader::new(&mut stream);
        reader.read(&mut request_buf).await?;
        
        //KNOWN_ADDRESSES hashtable request
        if u8::from(request_buf[0]) == KNOWN_ADDRESSES_REQUEST {
            let peers = get_peers().await;
            let mut response: [u8; 1] = [0; 1];
            for peer in peers {
                //println!("Writing");
                let peer = format!("{}{}", serde_json::to_string(&peer).unwrap(), '\n');
                reader.write(peer.as_bytes()).await?;
                //Await for response from the other end 
                tokio::select! {
                    _ = sleep(Duration::from_secs(1)) => {
                        println!("Connection closed");
                        break;
                    }
                    _ = reader.read(&mut response) => {
                        //Continue
                        //println!("Reading");
                    }
                }
            }
            stream.shutdown().await?;
        }
        //Info about peer request
        // if char::from(buf[0]) == PEER_INFO_REQUEST {
        //     let peer = get_this_peer().await.unwrap();
        //     let peer_string = to_string(&peer).unwrap();
        //     stream.write_all(peer_string.as_bytes()).await.unwrap();
        // }
        //println!("{}", char::from(buf[0]));
    }
    //Ok(())
}

pub async fn get_peer_addresses(socket: Arc<TcpListener>) -> std::io::Result<()> {
    let request: [u8; 1] = [KNOWN_ADDRESSES_REQUEST; 1];
    let mut response = [0; 1024];
    //Send request
    let (mut stream, _addr) = socket.accept().await?;
    stream.write_all(&request).await?;
    //Get response
    stream.read(&mut response).await?;
    println!("{}", str::from_utf8(&response).unwrap());
    
    Ok(())
}