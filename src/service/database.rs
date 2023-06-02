use std::{path::PathBuf, str::FromStr};

pub type Pool = sqlx::sqlite::SqlitePool;
pub type Transaction<'t> = sqlx::Transaction<'t, sqlx::Sqlite>;
pub type Error = sqlx::Error;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_url")]
    url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            migrations: Self::default_migrations(),
        }
    }
}

impl Config {
    fn default_url() -> String {
        "/etc/donos/database.db".to_string()
    }
}

impl Config {
    #[cfg(test)]
    pub fn test_env() -> Self {
        Self {
            url: String::from(":memory:"),
        }
    }

    pub async fn build(&self) -> Result<Pool, sqlx::Error> {
        tracing::debug!("connecting to database {:?}", self.url);
        let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&self.url)?.create_if_missing(true);
        sqlx::sqlite::SqlitePoolOptions::new()
            .min_connections(1)
            .connect_with(opts)
            .await
    }
}

impl Config {
    pub async fn migrate(&self, pool: &Pool) -> Result<(), Error> {
        tracing::debug!("running migrations");
        MIGRATOR.run(pool).await?;
        Ok(())
    }
}
