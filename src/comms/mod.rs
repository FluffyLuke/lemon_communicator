pub mod input;
pub mod output;
pub mod peer_data;
use std::sync::Arc;

use tokio::net::{UdpSocket, TcpSocket};
use crate::peer::Peer;

use self::{output::send, input::receive, peer_data::send_peer_data};

pub async fn start_server(this_peer: Peer) -> std::io::Result<()>{
    //Sockets for messages
    let socket_udp = UdpSocket::bind(format!("{}:{}", this_peer.get_address().get_addr(), this_peer.get_address().get_port())).await?;
    let socket_udp = Arc::new(socket_udp);
    let sock = Arc::clone(&socket_udp);
    let handle1 = tokio::spawn(send(sock));
    let sock = Arc::clone(&socket_udp);
    let handle2 = tokio::spawn(receive(sock));

    //Sockets for peer data
    let addr = format!("{}:{}", this_peer.get_address().get_addr(), this_peer.get_address().get_data_port()).parse().unwrap();
    let socket_tcp = TcpSocket::new_v4()?;
    socket_tcp.bind(addr)?;
    let stream = socket_tcp.listen(128)?;
    let handle3 = tokio::spawn(send_peer_data(stream));
    let _result1 = handle1.await?;
    let _result2 = handle2.await?;
    let _result3 = handle3.await?;
    Ok(())
}