use crate::prelude::Message;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct Sender {
    socket: Arc<UdpSocket>,
}

impl Sender {
    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }

    pub async fn send(&self, message: &Message) -> std::io::Result<()> {
        let Message {
            address,
            buffer,
            size,
        } = message;
        tracing::debug!("sending message to {:?}", address);
        self.socket.send_to(&buffer[0..*size], address).await?;
        Ok(())
    }
}
