use crate::prelude::Message;
use async_stream::stream;
use futures_core::stream::Stream;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct Receiver {
    socket: Arc<UdpSocket>,
}

impl Receiver {
    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }

    async fn receive(&self) -> std::io::Result<Message> {
        let mut buffer = [0u8; 512];
        let (size, address) = self.socket.recv_from(&mut buffer).await?;
        Ok(Message {
            address,
            buffer,
            size,
        })
    }

    pub fn into_stream(self) -> impl Stream<Item = Message> {
        stream! {
            while let Ok(message) = self.receive().await {
                tracing::debug!("received message from {:?}", message.address);
                yield message;
            }
        }
    }
}
