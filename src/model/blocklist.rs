use crate::service::database::{Error, Transaction};
use std::net::IpAddr;

pub async fn is_blocked<'t>(
    tx: &mut Transaction<'t>,
    _origin: &IpAddr,
    domain: &str,
) -> Result<bool, Error> {
    sqlx::query_scalar(
        r#"SELECT COUNT(id) > 0
FROM blocked_hostnames
WHERE domain = $1"#,
    )
    .bind(domain)
    .fetch_one(tx)
    .await
}
