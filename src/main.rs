// mod cmd;
// mod config;
// mod model;
// mod service;

// use clap::Parser;

use donos_server::prelude::Message;
use donos_server::receiver::Receiver;
use donos_server::sender::Sender;
use futures::stream::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tokio::net::UdpSocket;

fn init_logs() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{fmt, registry, EnvFilter};

    let _ = registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug,tower_http=debug", env!("CARGO_PKG_NAME")).into()
        }))
        .with(fmt::layer().with_ansi(cfg!(debug_assertions)))
        .try_init();
}

async fn do_something(msg: Message) -> Message {
    println!("received message from {:?}", msg.address);
    msg
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logs();

    // cmd::Args::parse().run().await;

    let socket = UdpSocket::bind("0.0.0.0:2056").await?;
    let socket = Arc::new(socket);

    let receiver = Receiver::new(socket.clone());
    let sender = Sender::new(socket);

    let stream = receiver
        .into_stream()
        .map(do_something)
        .buffer_unordered(64);

    tokio::pin!(stream);

    while let Some(item) = stream.next().await {
        if let Err(error) = sender.send(&item).await {
            eprintln!("couldn't send message to {:?}: {error:?}", item.address);
        }
    }

    Ok(())
}
