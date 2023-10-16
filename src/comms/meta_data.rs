use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};
use zerocopy::{FromBytes, AsBytes};
use zerocopy_derive::{FromBytes, AsBytes, Unaligned, FromZeroes};
use std::str;

use crate::peer::get_peers;

const KNOWN_ADDRESSES_REQUEST: u8 = 0;

pub async fn send_meta_data(socket: TcpListener) -> std::io::Result<()>{
    let mut buf  = [0; 1];
    loop {
        let (mut stream, _addr) = socket.accept().await?;
        stream.read(&mut buf).await?;
        
        //KNOWN_ADDRESSES hashtable request
        if u8::from_be_bytes(buf) == KNOWN_ADDRESSES_REQUEST {
            let a = get_peers().await;
            //END THIS
        }
    }
    Ok(())
}
