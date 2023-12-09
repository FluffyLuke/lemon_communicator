#[derive(Debug)]
pub struct Client {
    pub addr: std::net::SocketAddr,
    pub name: String,
}



impl Client {
    pub fn new(addr: std::net::SocketAddr, name: String) -> Client {
        Client {
            addr,
            name,
        }
    }
}