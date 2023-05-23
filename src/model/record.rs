use crate::service::database::{Error, Transaction};
use donos_proto::packet::record::Record;
use donos_proto::packet::QueryType;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

struct FoundRecord {
    query_type: u16,
    domain: String,
    host: String,
    ttl: u32,
    priority: u16,
}

impl FromRow<'_, SqliteRow> for FoundRecord {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            query_type: row.try_get(0)?,
            domain: row.try_get(1)?,
            host: row.try_get(2)?,
            ttl: row.try_get(3)?,
            priority: row.try_get(4)?,
        })
    }
}

impl TryFrom<FoundRecord> for Record {
    type Error = Error;

    fn try_from(value: FoundRecord) -> Result<Self, Self::Error> {
        let query_type = QueryType::from_num(value.query_type);
        match query_type {
            QueryType::A => {
                let addr = value.host.parse().map_err(|_err| Error::TypeNotFound {
                    type_name: "Ipv4Addr".into(),
                })?;
                Ok(Record::A {
                    domain: value.domain,
                    addr,
                    ttl: value.ttl,
                })
            }
            QueryType::AAAA => {
                let addr = value.host.parse().map_err(|_err| Error::TypeNotFound {
                    type_name: "Ipv6Addr".into(),
                })?;
                Ok(Record::AAAA {
                    domain: value.domain,
                    addr,
                    ttl: value.ttl,
                })
            }
            QueryType::CNAME => Ok(Record::CNAME {
                domain: value.domain,
                host: value.host,
                ttl: value.ttl,
            }),
            QueryType::MX => Ok(Record::MX {
                domain: value.domain,
                host: value.host,
                ttl: value.ttl,
                priority: value.priority,
            }),
            QueryType::NS => Ok(Record::NS {
                domain: value.domain,
                host: value.host,
                ttl: value.ttl,
            }),
            // this should be unreachable
            QueryType::Unknown(qtype) => Ok(Record::Unknown {
                domain: value.domain,
                qtype,
                data_len: 0,
                ttl: value.ttl,
            }),
        }
    }
}

pub async fn find<'t>(
    tx: &mut Transaction<'t>,
    qtype: QueryType,
    domain: &str,
) -> Result<Option<Record>, Error> {
    let record: Option<FoundRecord> = sqlx::query_as(
        r#"SELECT
    query_type,
    domain,
    host,
    max(created_at + ttl - UNIXEPOCH(), 0),
    priority
FROM dns_records
WHERE query_type = $1
AND domain = $2
AND created_at + ttl > UNIXEPOCH()"#,
    )
    .bind(qtype.into_num())
    .bind(domain)
    .fetch_optional(tx)
    .await?;
    if let Some(record) = record {
        Ok(Some(Record::try_from(record)?))
    } else {
        Ok(None)
    }
}

pub async fn persist<'t>(tx: &mut Transaction<'t>, record: &Record) -> Result<(), Error> {
    match record {
        Record::A { domain, addr, ttl } => {
            sqlx::query(
                r#"INSERT INTO dns_records (query_type, domain, host, ttl, created_at)
VALUES ($1, $2, $3, $4, UNIXEPOCH())
ON CONFLICT (query_type, domain) DO UPDATE SET host=$3, ttl=$4, created_at=UNIXEPOCH()"#,
            )
            .bind(QueryType::A.into_num())
            .bind(domain)
            .bind(&addr.to_string())
            .bind(ttl)
            .execute(tx)
            .await?;
        }
        _ => {
            tracing::debug!("not yes implemented")
        }
    }
    Ok(())
}
