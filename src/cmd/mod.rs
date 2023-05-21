pub mod dns;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
            Commands::Dns(inner) => inner.run(config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Dns(dns::Command),
}
