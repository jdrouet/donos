use std::net::SocketAddr;

use crate::service::database;

pub async fn resolve<'t>(tx: &mut database::Transaction<'t>) -> Result<(), database::Error> {
    todo!()
}

pub async fn is_blocked<'t>(
    tx: &mut database::Transaction<'t>,
    origin: &SocketAddr,
    qname: &str,
) -> Result<bool, database::Error> {
    Ok(qname == "google.fr")
}
