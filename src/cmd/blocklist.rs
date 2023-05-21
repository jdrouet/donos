use clap::Args;

/// Handle the blocklist in database
#[derive(Args, Debug)]
pub struct Command;

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

        let mut total_inserted = 0;
        let mut total_deleted = 0;

        let mut tx = database.begin().await.expect("couldn't start transaction");

        let loader = donos_blocklist_loader::BlocklistLoader::default();
        for (name, item) in config.blocklist.members {
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
}
