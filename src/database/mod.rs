use anyhow::Result;
use sqlx::SqlitePool;

mod sqlite;

#[derive(Debug, Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

pub trait Database: Clone {
    async fn tables(&self) -> Result<Vec<String>>;
    async fn schema(&self, table: &str) -> Result<String>;
    async fn columns(&self, table: &str) -> Result<Vec<String>>;
    async fn rows(&self, column: &str, table: &str) -> Result<Vec<Vec<serde_json::Value>>>;
}
