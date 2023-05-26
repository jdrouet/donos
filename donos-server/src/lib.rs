use futures::stream::StreamExt;
use prelude::Message;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub mod prelude;
pub mod receiver;
pub mod sender;

#[async_trait::async_trait]
pub trait Handler {
    async fn handle(&self, message: Message) -> Message;
}

pub struct UdpServer<H> {
    address: SocketAddr,
    handler: H,
}

impl<H: Handler> UdpServer<H> {
    pub fn new(address: SocketAddr, handler: H) -> Self {
        Self { address, handler }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let socket = UdpSocket::bind(self.address).await?;
        let socket = Arc::new(socket);

        let receiver = receiver::Receiver::new(socket.clone());
        let sender = sender::Sender::new(socket);

        let stream = receiver
            .into_stream()
            .map(|item| async { self.handler.handle(item).await })
            .buffer_unordered(64);

        tokio::pin!(stream);

        while let Some(item) = stream.next().await {
            if let Err(error) = sender.send(&item).await {
                tracing::error!("couldn't send message to {:?}: {error:?}", item.address);
            }
        }

        Ok(())
    }
}
