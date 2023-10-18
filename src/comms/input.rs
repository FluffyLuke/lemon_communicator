use std::sync::Arc;

use tokio::net::UdpSocket;

pub async fn receive(socket: Arc<UdpSocket>) -> std::io::Result<()>{
    let mut buf: [u8; 255] = [0; 255];
    println!("Listening for connections");
    loop {
        socket.recv(&mut buf).await.unwrap();
        println!("got something: {}", String::from_utf8_lossy(&buf));
    }
}

