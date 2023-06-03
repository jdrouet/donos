use donos_blocklist_loader::BlocklistKind;
use sqlx::{Pool, Sqlite};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    net::SocketAddr,
};

use crate::service::database::Transaction;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BlocklistItem {
    pub url: String,
    pub kind: BlocklistKind,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub inner: BTreeMap<String, BlocklistItem>,
}

impl Config {
    pub fn build(self, database: Pool<Sqlite>) -> DatabaseBlocklistService {
        DatabaseBlocklistService::new(self.inner, database)
    }
}

#[async_trait::async_trait]
pub trait BlocklistService {
    async fn is_blocked(&self, origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>>;
    async fn import(&self) -> Result<(u64, u64), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct DatabaseBlocklistService {
    #[allow(dead_code)]
    database: Pool<Sqlite>,
    items: BTreeMap<String, BlocklistItem>,
}

impl DatabaseBlocklistService {
    pub fn new(items: BTreeMap<String, BlocklistItem>, database: Pool<Sqlite>) -> Self {
        Self { items, database }
    }
}

async fn import_list<'t>(
    tx: &mut Transaction<'t>,
    url: &str,
    description: &str,
    hash: &str,
    domains: HashSet<String>,
) -> Result<(u64, u64), sqlx::Error> {
    // check if exists with same hash
    let exists: bool = sqlx::query_scalar(
        r#"SELECT count(id) > 0
FROM blocklists
WHERE url = $1 AND last_refresh_hash = $2"#,
    )
    .bind(url)
    .bind(hash)
    .fetch_one(&mut *tx)
    .await?;
    // The same hash as already been imported, we can pass
    if exists {
        return Ok((0, 0));
    }
    // upsert the blocklist
    let blocklist_id: u32 = sqlx::query_scalar(
        r#"INSERT INTO blocklists (url, description, created_at, last_refresh_at, last_refresh_hash)
VALUES ($1, $2, UNIXEPOCH(), UNIXEPOCH(), $3)
ON CONFLICT (url) DO UPDATE SET last_refresh_at = UNIXEPOCH(), last_refresh_hash = $3
RETURNING id"#,
    )
    .bind(url)
    .bind(description)
    .bind(hash)
    .fetch_one(&mut *tx)
    .await?;

    // create a temporary table
    sqlx::query("CREATE TEMPORARY TABLE import_blocked_domains (domain TEXT UNIQUE NOT NULL)")
        .execute(&mut *tx)
        .await?;

    // insert domains in temporary table
    for item in domains {
        sqlx::query("INSERT INTO import_blocked_domains (domain) VALUES ($1)")
            .bind(&item)
            .execute(&mut *tx)
            .await?;
    }

    // removing entries that are not there anymore
    let deleted = sqlx::query("DELETE FROM blocked_domains WHERE domain NOT IN (SELECT domain FROM import_blocked_domains) AND blocklist_id = $1")
        .bind(blocklist_id)
        .execute(&mut *tx).await?;

    // moving entries from temporary table
    let inserted = sqlx::query(
        r#"INSERT INTO blocked_domains (blocklist_id, domain, created_at)
SELECT $1 AS blocklist_id, domain, UNIXEPOCH() AS created_at
FROM import_blocked_domains
WHERE true
ON CONFLICT (blocklist_id, domain) DO NOTHING"#,
    )
    .bind(blocklist_id)
    .execute(&mut *tx)
    .await?;

    // drop the temporary table
    sqlx::query("DROP TABLE import_blocked_domains")
        .execute(&mut *tx)
        .await?;

    Ok((inserted.rows_affected(), deleted.rows_affected()))
}

#[async_trait::async_trait]
impl BlocklistService for DatabaseBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        let exists: bool =
            sqlx::query_scalar("SELECT count(id) > 0 FROM blocked_domains WHERE domain = ?")
                .bind(domain)
                .fetch_one(&self.database)
                .await?;
        Ok(exists)
    }

    #[tracing::instrument(skip(self))]
    async fn import(&self) -> Result<(u64, u64), Box<dyn Error>> {
        let mut tx = self.database.begin().await?;

        let mut total_inserted = 0;
        let mut total_deleted = 0;

        let loader = donos_blocklist_loader::BlocklistLoader::default();
        for (name, item) in self.items.iter() {
            tracing::debug!("start loading {name:?}");
            match loader.load(&item.url, item.kind).await {
                Ok(result) => {
                    tracing::debug!(
                        "loaded blocklist {name:?} with {} domains and hash {}",
                        result.entries.len(),
                        result.hash
                    );
                    let description = format!("{name} blocklist of {:?} kind", item.kind);
                    let (inserted, deleted) = import_list(
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
        Ok((total_inserted, total_deleted))
    }
}

#[derive(Debug, Default)]
pub struct MemoryBlocklistService {
    inner: std::collections::HashSet<String>,
}

#[cfg(test)]
impl MemoryBlocklistService {
    pub fn with_domain<D: Into<String>>(mut self, domain: D) -> Self {
        self.inner.insert(domain.into());
        self
    }
}

#[async_trait::async_trait]
impl BlocklistService for MemoryBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        Ok(self.inner.contains(domain))
    }

    #[tracing::instrument(skip(self))]
    async fn import(&self) -> Result<(u64, u64), Box<dyn Error>> {
        Ok((0, 0))
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::blocklist::BlocklistService;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn address() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 56))
    }

    #[tokio::test]
    async fn database_service_should_block() {
        crate::init_logs();

        let database = crate::service::database::Config::test_env()
            .build()
            .await
            .unwrap();
        crate::service::database::migrate(&database).await.unwrap();

        let _: u32 = sqlx::query_scalar(
            "insert into blocked_domains (domain, created_at) values (?, UNIXEPOCH()) returning id",
        )
        .bind("facebook.com")
        .fetch_one(&database)
        .await
        .unwrap();

        let addr = address();

        let service = super::DatabaseBlocklistService::new(Default::default(), database);

        let is_blocked = service.is_blocked(&addr, "facebook.com").await.unwrap();
        assert!(is_blocked);
        let is_blocked = service.is_blocked(&addr, "perdu.com").await.unwrap();
        assert!(!is_blocked);
    }
}
