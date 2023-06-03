use clap::Args;

use crate::repository::blocklist::BlocklistService;

/// Handle the blocklist in database
#[derive(Args, Debug)]
pub struct Command {}

impl Command {
    pub async fn run(self, config: crate::config::Config) {
        let database = config
            .database
            .build()
            .await
            .expect("unable to connect to database");
        crate::service::database::migrate(&database)
            .await
            .expect("unable to migrate the database");

        let blocklist = config.blocklists.build(database);
        match blocklist.import().await {
            Ok((inserted, deleted)) => {
                tracing::info!(
                    "inserted {inserted} new domains and deleted {deleted} existing domains"
                );
            }
            Err(err) => {
                tracing::error!("couldn't import blocklists: {err:?}");
            }
        }
    }
}
