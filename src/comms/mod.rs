pub mod input;
pub mod output;
pub mod peer_data;
use std::sync::Arc;

use tokio::net::{UdpSocket, TcpSocket};
use crate::peer::Peer;

use self::{output::send, input::receive, peer_data::send_peer_data};

pub async fn start_server(this_peer: Peer) -> std::io::Result<()>{
    let addr = this_peer.get_addr_and_data_port().parse().unwrap();
    let socket_tcp = TcpSocket::new_v4()?;
    socket_tcp.bind(addr)?;
    let socket_udp = UdpSocket::bind(this_peer.get_addr_and_port()).await?;
    let socket_udp = Arc::new(socket_udp);

    let tcp_listener = Arc::new(socket_tcp.listen(128)?);
    let sock_udp = Arc::clone(&socket_udp);
    let stream = Arc::clone(&tcp_listener);
    let handle1 = tokio::spawn(send(sock_udp, stream));
    let sock_udp = Arc::clone(&socket_udp);
    let handle2 = tokio::spawn(receive(sock_udp));
    let stream = Arc::clone(&tcp_listener);
    let handle3 = tokio::spawn(send_peer_data(stream));
    let _result1 = handle1.await?;
    let _result2 = handle2.await?;
    let _result3 = handle3.await?;
    Ok(())
}