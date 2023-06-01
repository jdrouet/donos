use clap::Args;
use donos_server::UdpServer;
use std::sync::Arc;

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod handler;

/// Starts the DNS server, the core of the machine
#[derive(Args, Debug)]
pub struct Command;

impl Command {
    pub async fn run(&self, config: crate::config::Config) {
        tracing::info!("preparing dns server");
        // let database = config
        //     .database
        //     .build()
        //     .await
        //     .expect("unable to connect database");
        // config
        //     .database
        //     .migrate(&database)
        //     .await
        //     .expect("unable to run database migration");
        let cache_service = config
            .cache
            .build()
            .await
            .expect("unable to build cache service");
        let lookup_service = config
            .lookup
            .build()
            .await
            .expect("unable to build lookup service");
        // let handler = DnsHandler::new(database, lookup);
        let blocklist_service = crate::repository::blocklist::MockBlocklistService::default();
        let handler = handler::DnsHandler::new(
            Arc::new(blocklist_service),
            Arc::new(cache_service),
            Arc::new(lookup_service),
        );

        let address = config.dns.address();
        UdpServer::new(address, handler)
            .run()
            .await
            .expect("unable to run udp server")
    }
}
