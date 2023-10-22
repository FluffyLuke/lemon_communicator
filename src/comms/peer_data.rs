use serde_json::to_string;
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};
use std::{str, string, sync::Arc};

use crate::peer::get_peers;

const KNOWN_ADDRESSES_REQUEST: u8 = 1;
const PEER_INFO_REQUEST: u8 = 1;

pub async fn send_peer_data(socket: Arc<TcpListener>) -> std::io::Result<()> {
    let mut buf  = [0; 1];
    loop {
        let (mut stream, _addr) = socket.accept().await?;
        stream.read(&mut buf).await?;
        
        //KNOWN_ADDRESSES hashtable request
        if u8::from(buf[0]) == KNOWN_ADDRESSES_REQUEST {
            let peers = get_peers().await;
            let peers_string = to_string(&peers).unwrap();
            stream.write_all(peers_string.as_bytes()).await?;
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