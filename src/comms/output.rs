use std::{sync::Arc, fmt, net::TcpStream, io::Write, ptr::metadata};

use tokio::{net::{UdpSocket, TcpListener}, stream};

use crate::{peer::{get_peers, Peer}, myio::{get_input, get_input_with_message, get_input_parsed, InputError}};

use super::peer_data::PEER_INFO_REQUEST;

pub async fn send(udp_socket: Arc<UdpSocket>, tcp_listener: Arc<TcpListener>) -> std::io::Result<()>{
    //let ip_regex: regex::Regex = Regex::new(r"[0-9]+(?:\.[0-9]+){3}:[0-9]+").unwrap();
    if get_peers().await.capacity() == 0 {
        println!("No peers found");
        return Ok(());
    }
    loop {
        let mut chosen_peer;
        loop {
            chosen_peer = choose_peer().await;
            if let Err(e) = chosen_peer {
                println!("{e}");
                continue;
            }
            break;
        };
        let chosen_peer = chosen_peer.unwrap();
        
        //Actions 
        println!("SAY WHAT DO TO:");
        println!("send message = {}", SEND_MESSAGE);
        println!("send message = {}", GET_PEER_HASHTABLE);
        println!("get peers = NIMA");

        let input = get_input_parsed::<u8>();
        if let Err(_) = input {
            println!("Wrong option");
            continue;
        }
        let input = input.unwrap();
        match input {
            0 => { 
                let result = send_message(chosen_peer, &*udp_socket).await;
                if let Err(e) = result {
                    println!("{e}");
                }
            },
            _ => println!("Wrong option!")
        }
    }

}

async fn choose_peer() -> Result<Peer, ChoosingPeerError> {
    //Get peers
    let peers = get_peers().await;
    //Select peer from peers
    println!("Select peer:");
    for (i, peer) in peers.iter().enumerate() {
        println!("{}:{}", i, peer);
    }
    let input = get_input()
        .map_err(|e| ChoosingPeerError::InputError(e))? //TODO Handle the input error
        .parse::<usize>()
        .map_err(|_| ChoosingPeerError::ParsingError)?;
    let mut chosen_peer: Option<Peer> = None;
    if !(0..peers.len()).contains(&input) {
        return Err(ChoosingPeerError::WrongOption);
    }
    for (i, peer) in peers.iter().enumerate() {
        if i == input as usize {
            chosen_peer = Some(peer.clone());
            println!("Peer chosen: {}", &chosen_peer.clone().unwrap());
        }
    }
    if let None = chosen_peer {
        return Err(ChoosingPeerError::PeerNotFound);
    }
    let chosen_peer = chosen_peer.unwrap();
    Ok(chosen_peer)
}

#[derive(Debug)]
enum ChoosingPeerError {
    WrongOption,
    ParsingError,
    PeerNotFound,
    InputError(InputError)
}

impl fmt::Display for ChoosingPeerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChoosingPeerError::WrongOption => {
                write!(f, "Provided wrong option")
            }
            ChoosingPeerError::ParsingError => {
                write!(f, "Option was not a number")
            }
            ChoosingPeerError::PeerNotFound => {
                write!(f, "Peer not found in the saved peers")
            }
            ChoosingPeerError::InputError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

const SEND_MESSAGE: u8 = 0;
async fn send_message(chosen_peer: Peer, udp_socket: &UdpSocket) -> Result<(), SendMessageError>{
    let message = get_input_with_message("Provide message: ")
    .map_err(|e| SendMessageError::InputError(e))?;
    println!("sent string: {message}");
    let target_peer = chosen_peer.clone();
    let target_address = target_peer.get_address();
    println!("Target: {}:{}", target_address.get_addr(), target_address.get_port());
    udp_socket.send_to(message.as_bytes(), format!("{}:{}", target_address.get_addr(), target_address.get_port()))
    .await
    .map_err(|e| SendMessageError::SendError(e))?;
    Ok(())
}

enum SendMessageError {
    SendError(std::io::Error),
    InputError(InputError),
}

impl fmt::Display for SendMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SendError(e) => {
                write!(f, "Cannot send a message: {}", e)
            }
            Self::InputError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

const GET_PEER_HASHTABLE: u8 = 1;
async fn get_peer_hashtable(chosen_peer: Peer) -> Result<(), GetPeerHashtableError>{
    let mut stream = TcpStream::connect(chosen_peer.get_address().get_addr())
    .map_err(|e| GetPeerHashtableError::IOError(e))?;
    let message = &[PEER_INFO_REQUEST];
    stream.write_all(message);
    stream.read_exact(buf)
    Ok(())
}

enum GetPeerHashtableError {
    IOError(std::io::Error)
}

impl fmt::Display for GetPeerHashtableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "{e}")
            }
        }
    }
}