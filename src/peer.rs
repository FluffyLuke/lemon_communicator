use std::{error::Error, fmt::{self, format}, collections::HashSet, sync::Arc, thread::{ThreadId, Thread}};

use local_ip_address::local_ip;
use regex::Regex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::ops::DerefMut;

lazy_static! {
    static ref KNOWN_ADDRESSES: Arc<Mutex<HashSet<Peer>>> = Arc::new(Mutex::new(HashSet::new()));
}

pub async fn get_peers() -> HashSet<Peer> {
    let addresses = KNOWN_ADDRESSES.lock().await;
    let known_addresses = addresses.clone();
    known_addresses
}

pub async fn add_peer(peer: Peer) -> bool {
    let mut addresses = KNOWN_ADDRESSES.lock().await;
    addresses.insert(peer)
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    address: Address
}

impl Peer {
    pub fn new(address: Address) -> Peer {
        Peer {
            address
        }
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = format!("address: {}\ntcp port: {}\nudp port: {}",
            self.address.addr, self.address.data_port, self.address.port
        );
        write!(f, "{display}")
    }
}
#[derive(Eq, Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    addr: String,
    port: u32,
    data_port: u32,
}

impl Address {
    pub fn new(addr: String, port: u32, data_port: u32) -> Result<Address, AddressError> {
        let ip_regex: regex::Regex = Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$").unwrap();
        if !ip_regex.is_match(&addr) {
            return Err(AddressError::WrongAddr)
        }
        Ok(Address {
            addr,
            port,
            data_port,
        })
    }

    pub fn get_port(&self) -> u32 {
        self.port
    }
    pub fn get_data_port(&self) -> u32 {
        self.data_port
    }
    pub fn get_addr(&self) -> &str {
        &self.addr
    }
}

#[derive(Debug)]
pub enum AddressError {
    WrongAddr,
    WrongPort,
}
impl Error for AddressError {}
impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}