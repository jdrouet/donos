use donos_blocklist_loader::BlocklistKind;
use sqlx::{Pool, Sqlite};
use std::{collections::BTreeMap, error::Error, net::SocketAddr};

#[derive(Debug, serde::Deserialize)]
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
    pub fn build(&self, database: Pool<Sqlite>) -> DatabaseBlocklistService {
        DatabaseBlocklistService::new(database)
    }
}

#[async_trait::async_trait]
pub trait BlocklistService {
    async fn is_blocked(&self, origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct DatabaseBlocklistService {
    #[allow(dead_code)]
    database: Pool<Sqlite>,
}

impl DatabaseBlocklistService {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
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

        let service = super::DatabaseBlocklistService::new(database);

        let is_blocked = service.is_blocked(&addr, "facebook.com").await.unwrap();
        assert!(is_blocked);
        let is_blocked = service.is_blocked(&addr, "perdu.com").await.unwrap();
        assert!(!is_blocked);
    }
}
