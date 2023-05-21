use crate::service::database::{Error, Transaction};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use std::{collections::HashSet, net::IpAddr};

pub async fn is_blocked<'t>(
    tx: &mut Transaction<'t>,
    _origin: &IpAddr,
    domain: &str,
) -> Result<bool, Error> {
    sqlx::query_scalar(
        r#"SELECT COUNT(id) > 0
FROM blocked_domains
WHERE domain = $1"#,
    )
    .bind(domain)
    .fetch_one(tx)
    .await
}

pub async fn import<'t>(
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

#[derive(Debug)]
pub struct BlocklistReport {
    pub id: u32,
    pub url: String,
    pub description: String,
    pub created_at: u32,
    pub last_refresh_at: u32,
    pub last_refresh_hash: String,
    pub domain_count: u32,
}

impl FromRow<'_, SqliteRow> for BlocklistReport {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            url: row.try_get(1)?,
            description: row.try_get(2)?,
            created_at: row.try_get(3)?,
            last_refresh_at: row.try_get(4)?,
            last_refresh_hash: row.try_get(5)?,
            domain_count: row.try_get(6)?,
        })
    }
}

pub async fn reports<'t>(tx: &mut Transaction<'t>) -> Result<Vec<BlocklistReport>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT
    blocklists.id,
    blocklists.url,
    blocklists.description,
    blocklists.created_at,
    blocklists.last_refresh_at,
    blocklists.last_refresh_hash,
    count(blocked_domains.domain)
FROM blocklists
JOIN blocked_domains ON blocked_domains.blocklist_id = blocklists.id
GROUP BY blocklists.id"#,
    )
    .fetch_all(tx)
    .await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn should_import_non_existing() {
        let pool = crate::service::database::Config::default()
            .build()
            .await
            .unwrap();
        crate::service::database::migrate(&pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();

        let (inserted, deleted) = super::import(
            &mut tx,
            "https://example.com/blocklist.txt",
            "foo",
            "hash",
            ["google.com".to_owned(), "duckduckgo.com".to_owned()]
                .into_iter()
                .collect(),
        )
        .await
        .unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn should_import_with_same_hash() {
        let pool = crate::service::database::Config::default()
            .build()
            .await
            .unwrap();
        crate::service::database::migrate(&pool).await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        let (inserted, deleted) = super::import(
            &mut tx,
            "https://example.com/blocklist.txt",
            "foo",
            "hash",
            ["google.com".to_owned(), "duckduckgo.com".to_owned()]
                .into_iter()
                .collect(),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(deleted, 0);

        let mut tx = pool.begin().await.unwrap();
        let (inserted, deleted) = super::import(
            &mut tx,
            "https://example.com/blocklist.txt",
            "foo",
            "hash",
            ["google.com".to_owned(), "duckduckgo.com".to_owned()]
                .into_iter()
                .collect(),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted, 0);
        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn should_import_with_different_hash() {
        let pool = crate::service::database::Config::default()
            .build()
            .await
            .unwrap();
        crate::service::database::migrate(&pool).await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        let (inserted, deleted) = super::import(
            &mut tx,
            "https://example.com/blocklist.txt",
            "foo",
            "hash",
            ["google.com".to_owned(), "duckduckgo.com".to_owned()]
                .into_iter()
                .collect(),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(deleted, 0);

        let mut tx = pool.begin().await.unwrap();
        let (inserted, deleted) = super::import(
            &mut tx,
            "https://example.com/blocklist.txt",
            "foo",
            "other hash",
            [
                "foo.com".to_owned(),
                "duckduckgo.com".to_owned(),
                "bar.com".to_owned(),
            ]
            .into_iter()
            .collect(),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted, 2);
        assert_eq!(deleted, 1);
    }
}
