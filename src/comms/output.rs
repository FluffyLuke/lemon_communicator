use std::sync::Arc;

use tokio::net::{UdpSocket, TcpListener};

use crate::{peer::{get_peers, Peer}, myio::{get_input, get_input_with_message, get_input_parsed}};

pub async fn send(socket: Arc<UdpSocket>, data_socket: Arc<TcpListener>) -> std::io::Result<()>{
    //let ip_regex: regex::Regex = Regex::new(r"[0-9]+(?:\.[0-9]+){3}:[0-9]+").unwrap();
    if get_peers().await.capacity() == 0 {
        println!("No peers found");
        return Ok(());
    }
    loop {
        let peers = get_peers().await;
        peers.capacity();
        println!("Select peer:");
        for (i, peer) in peers.iter().enumerate() {
            println!("{}:{}", i, peer);
        }
        let input = get_input().unwrap().parse::<usize>();
        if let Err(_) = input {
            println!("Provided wrong peer");
            continue;
        }
        let input = input.unwrap();
        let len = peers.len();
        if input > len-1 {
            println!("Provided wrong peer");
            continue;
        }
        let mut chosen_peer: Option<Peer> = None;
        for (i, peer) in peers.iter().enumerate() {
            if i == input as usize {
                chosen_peer = Some(peer.clone());
                println!("Peer chosen: {}", &chosen_peer.clone().unwrap());
            }
        }
        if let None = chosen_peer {
            println!("Peer not found");
            continue;
        }
        let chosen_peer = chosen_peer.unwrap();
        
        //Actions 
        println!("SAY WHAT DO TO:");
        println!("send message = {}", SEND_MESSAGE);
        println!("get peers = NIMA");

        let input = get_input_parsed::<u8>();
        if let Err(_) = input {
            println!("Wrong option");
            continue;
        }
        let input = input.unwrap();
        match input {
            0 => { send_message(chosen_peer, &*socket).await },
            _ => println!("Wrong option!")
        }
    }

}

const SEND_MESSAGE: u8 = 0;
async fn send_message(chosen_peer: Peer, udp_socket: &UdpSocket) {
    let message = get_input_with_message("Provide message: ").unwrap();
        println!("sent string: {message}");
        let target_peer = chosen_peer.clone();
        let target_address = target_peer.get_address();
        println!("Target: {}:{}", target_address.get_addr(), target_address.get_port());
        udp_socket.send_to(message.as_bytes(), format!("{}:{}", target_address.get_addr(), target_address.get_port())).await.unwrap();
}