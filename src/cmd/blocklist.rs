use clap::{Args, Subcommand};

/// Handle the blocklist in database
#[derive(Args, Debug)]
pub struct Command {
    #[command(subcommand)]
    action: Action,
}

impl Command {
    pub async fn run(self, config: crate::config::Config) {
        self.action.run(config).await
    }
}

#[derive(Debug, Subcommand)]
enum Action {
    Sync,
    Print,
}

impl Action {
    async fn run_sync(self, config: crate::config::Config) {
        let database = config
            .database
            .build()
            .await
            .expect("unable to connect to database");
        crate::service::database::migrate(&database)
            .await
            .expect("unable to migrate the database");

        let mut total_inserted = 0;
        let mut total_deleted = 0;

        let mut tx = database.begin().await.expect("couldn't start transaction");

        let loader = donos_blocklist_loader::BlocklistLoader::default();
        for (name, item) in config.blocklists.inner {
            tracing::debug!("start loading {name:?}");
            match loader.load(&item.url, item.kind).await {
                Ok(result) => {
                    tracing::debug!(
                        "loaded blocklist {name:?} with {} domains and hash {}",
                        result.entries.len(),
                        result.hash
                    );
                    let description = format!("{name} blocklist of {:?} kind", item.kind);
                    let (inserted, deleted) = crate::model::blocklist::import(
                        &mut tx,
                        &item.url,
                        &description,
                        &result.hash,
                        result.entries,
                    )
                    .await
                    .expect("couldn't import blocklist");
                    tracing::debug!("blocklist {name:?} inserted {inserted} new domains and deleted {deleted} existing domains");
                    total_inserted += inserted;
                    total_deleted += deleted;
                }
                Err(error) => tracing::warn!("unable to load blocklist {name:?}: {error:?}"),
            };
        }
        tx.commit().await.expect("couldn't commit changes");
        tracing::info!(
            "inserted {total_inserted} new domains and deleted {total_deleted} existing domains"
        );
    }

    async fn run_print(self, config: crate::config::Config) {
        let database = config
            .database
            .build()
            .await
            .expect("unable to connect to database");
        crate::service::database::migrate(&database)
            .await
            .expect("unable to migrate the database");

        let mut tx = database.begin().await.expect("couldn't start transaction");

        let reports = crate::model::blocklist::reports(&mut tx)
            .await
            .expect("unable to fetch reports");

        if reports.is_empty() {
            tracing::info!("there is no blocklist in the database");
        } else {
            for item in reports {
                tracing::info!(
                    "blocklist {} ({}) contains {} domains",
                    item.id,
                    item.description,
                    item.domain_count
                );
            }
        }
    }

    async fn run(self, config: crate::config::Config) {
        match self {
            Self::Sync => self.run_sync(config).await,
            Self::Print => self.run_print(config).await,
        }
    }
}
