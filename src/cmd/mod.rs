pub mod blocklist;
pub mod dns;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "./donos.toml")]
    config: PathBuf,
    #[command(subcommand)]
    inner: Commands,
}

impl Args {
    pub async fn run(self) {
        let config = crate::config::Config::load(&self.config);
        match self.inner {
            Commands::Blocklist(inner) => inner.run(config).await,
            Commands::Dns(inner) => inner.run(config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    Blocklist(blocklist::Command),
    Dns(dns::Command),
}
