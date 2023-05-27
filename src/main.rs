mod cmd;
mod config;
mod model;
mod service;

use clap::Parser;

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logs();

    cmd::Args::parse().run().await;

    Ok(())
}
