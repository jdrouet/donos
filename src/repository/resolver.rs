use crate::service::database;
use std::net::SocketAddr;

pub async fn is_blocked<'t>(
    _tx: &mut database::Transaction<'t>,
    _origin: &SocketAddr,
    qname: &str,
) -> Result<bool, database::Error> {
    Ok(qname == "google.fr")
}
