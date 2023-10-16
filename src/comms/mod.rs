pub mod input;
pub mod output;
pub mod meta_data;
use std::sync::Arc;

use tokio::net::{UdpSocket, TcpSocket};
use crate::Args;
use self::{output::send, input::receive, meta_data::send_meta_data};

pub async fn start_server(args: Args) -> std::io::Result<()>{
    //Sockets for messages
    let socket_udp = UdpSocket::bind(format!("127.0.0.1:{}", args.port)).await?;
    let socket_udp = Arc::new(socket_udp);

    //Sockets for metadata
    let addr = format!("127.0.0.1:{}", args.port_meta).parse().unwrap();
    let socket_tcp = TcpSocket::new_v4()?;
    socket_tcp.bind(addr)?;
    let stream = socket_tcp.listen(128)?;
    let handle3 = tokio::spawn(send_meta_data(stream));

    let sock = Arc::clone(&socket_udp);
    let handle1 = tokio::spawn(send(sock));
    let sock = Arc::clone(&socket_udp);
    let handle2 = tokio::spawn(receive(sock));
    let sock = Arc::clone(&socket_udp);
    let _result1 = handle1.await?;
    let _result2 = handle2.await?;
    let _result3 = handle3.await?;
    Ok(())
}