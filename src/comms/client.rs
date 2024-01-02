use std::sync::atomic::{AtomicU64, Ordering};
use std::vec::Vec;

use serde::{Serialize, Deserialize};
use serde::ser::SerializeSeq;

#[derive(Debug, Clone, Serialize, Deserialize, std::cmp::Eq, PartialEq)]
pub struct Client {
    pub id: u64,
    pub addr: std::net::SocketAddr,
    pub name: String,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String) -> Client {
        Client {
            id: ID.fetch_add(1, Ordering::SeqCst),
            addr,
            name,
        }
    }
}

// pub struct ClientVec(Vec<Client>);

// impl Serialize for ClientVec {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: serde::Serializer {
//         let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
//         for c in self.0 {
//             seq.serialize_element(&c.id)?;
//             seq.serialize_element(&c.addr)?;
//             seq.serialize_element(&c.name)?;
//         }
//         seq.end()
//     }
// }
