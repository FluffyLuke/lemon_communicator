use serde_json::to_string;
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};
use std::str;

use crate::peer::get_peers;

const KNOWN_ADDRESSES_REQUEST: char = '1';
const PEER_INFO_REQUEST: char = '2';

pub async fn send_peer_data(socket: TcpListener) -> std::io::Result<()>{
    let mut buf  = [0; 1];
    loop {
        let (mut stream, _addr) = socket.accept().await?;
        stream.read(&mut buf).await?;
        
        //KNOWN_ADDRESSES hashtable request
        if char::from(buf[0]) == KNOWN_ADDRESSES_REQUEST {
            let peers = get_peers().await;
            let peers_string = to_string(&peers).unwrap();
            stream.write_all(peers_string.as_bytes()).await.unwrap();
        }
        //Info about peer request
        // if char::from(buf[0]) == PEER_INFO_REQUEST {
        //     let peer = get_this_peer().await.unwrap();
        //     let peer_string = to_string(&peer).unwrap();
        //     stream.write_all(peer_string.as_bytes()).await.unwrap();
        // }
        println!("{}", char::from(buf[0]));
    }
    //Ok(())
}