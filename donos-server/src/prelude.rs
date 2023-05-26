use std::net::SocketAddr;

pub struct Message {
    pub address: SocketAddr,
    pub buffer: [u8; 512],
    pub size: usize,
}
