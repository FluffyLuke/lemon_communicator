use std::sync::Arc;

use regex::Regex;
use tokio::net::UdpSocket;

use crate::{peer::{get_peers, Peer}, myio::{get_input, get_input_with_message}, comms::input};

pub async fn send(socket: Arc<UdpSocket>) -> std::io::Result<()>{
    //let ip_regex: regex::Regex = Regex::new(r"[0-9]+(?:\.[0-9]+){3}:[0-9]+").unwrap();
    let mut chosen_peer: Option<Peer> = None;
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
        let input = get_input()?.parse::<usize>();
        if let Err(_) = input {
            println!("Provided wrong peer");
            continue;
        }
        let input = input.unwrap();
        let capacity = peers.capacity();
        if input <= 0 && input > capacity {
            continue;
        }
        for (i, peer) in peers.iter().enumerate() {
            if i == input as usize {
                chosen_peer = Some(peer.clone());
                println!("Peer chosen: {}", &chosen_peer.clone().unwrap());
            }
        }
        let message = get_input_with_message("Provide message: ").unwrap();
        println!("sent string: {message}");
        socket.send_to(message.as_bytes(),&chosen_peer.clone().unwrap().get_address()).await.unwrap();
    }

}