
use clap::Parser;
use comms::start_server;
use local_ip_address::local_ip;
use peer::{Peer, Address};

mod myio;
mod peer;
mod comms;



#[tokio::main]
async fn main() -> std::io::Result<()> {
    peer::add_peer(Peer::new(Address::new("192.168.11.11".to_string(), 1236, 1237).unwrap())).await;
    
    
    let args = Args::parse();
    let addr = local_ip().unwrap().to_string();
    println!("Your ip address is -> {}", addr);
    let this_peer = Peer::new(Address::new(
        addr,
        args.port,
        args.meta_port
    ).unwrap());
    peer::add_peer(this_peer.clone()).await;
    println!("Your peer: {}", this_peer);
    println!("Starting server");
    start_server(this_peer).await?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    //Port for udp socket
    #[arg(short, long, default_value_t = 1234)]
    port: u32,
    //Port for tcp socket
    #[arg(short, long, default_value_t = 1235)]
    meta_port: u32,
}