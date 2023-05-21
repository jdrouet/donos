use std::path::Path;

pub type Pool = sqlx::sqlite::SqlitePool;
pub type Transaction<'t> = sqlx::Transaction<'t, sqlx::Sqlite>;
pub type Error = sqlx::Error;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_url")]
    url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
        }
    }
}

impl Config {
    fn default_url() -> String {
        "sqlite::memory:".to_string()
    }
}

impl Config {
    pub async fn build(self) -> Result<Pool, sqlx::Error> {
        tracing::debug!("connecting to database {:?}", self.url);
        sqlx::sqlite::SqlitePoolOptions::new()
            .min_connections(1)
            .connect(&self.url)
            .await
    }
}

pub async fn migrate(pool: &Pool) -> Result<(), Error> {
    tracing::debug!("running migrations");
    let migrator = sqlx::migrate::Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    Ok(())
}
