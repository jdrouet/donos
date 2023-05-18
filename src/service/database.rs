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
            .connect(&self.url)
            .await
    }
}
