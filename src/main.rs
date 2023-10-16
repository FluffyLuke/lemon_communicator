use std::sync::Arc;

use clap::Parser;
use comms::{output::send, input::receive, start_server};
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
    start_server(args).await?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    //Port for udp socket
    #[arg(short, long, default_value_t = 1234)]
    port: u16,
    //Port for tcp socket
    #[arg(short, long, default_value_t = 1235)]
    port_meta: u32,
}