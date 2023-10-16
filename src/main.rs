use std::sync::Arc;

use clap::Parser;
use comms::{output::send, input::receive};
use peer::{Peer, Address};
use regex::Regex;
use tokio::net::UdpSocket;

mod myio;
mod peer;
mod comms;

use myio::{get_input, get_input_with_message};

use crate::peer::get_peers;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    peer::add_peer(Peer::new(Address::new("127.0.0.1".to_string(), 1234).unwrap())).await;
    peer::add_peer(Peer::new(Address::new("127.0.0.1".to_string(), 1235).unwrap())).await;
    let args = Args::parse();
    let socket = UdpSocket::bind(format!("127.0.0.1:{}", args.port)).await?;
    let socket = Arc::new(socket);

    let sock = Arc::clone(&socket);
    let _handle1 = tokio::spawn(send(sock));
    let sock = Arc::clone(&socket);
    let handle2 = tokio::spawn(receive(sock));
    handle2.await?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1234)]
    port: u16,
    //#[arg(short, long)]
    //first_peer_port: u16,
}
