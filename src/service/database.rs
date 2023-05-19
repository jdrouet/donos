use std::path::Path;

pub type Pool = sqlx::sqlite::SqlitePool;
pub type Transaction<'t> = sqlx::Transaction<'t, sqlx::Sqlite>;
pub type Error = sqlx::Error;

pub struct Config {
    url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL").unwrap_or_else(|_| String::from("sqlite::memory:")),
        }
    }

    pub async fn build(self) -> Result<Pool, sqlx::Error> {
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
