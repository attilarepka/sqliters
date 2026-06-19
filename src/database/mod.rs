use anyhow::Result;
use sqlx::SqlitePool;

mod sqlite;

#[derive(Debug, Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

pub trait Database: Clone {
    async fn tables(&self) -> Result<Vec<String>>;
    async fn table_schema(&self, table: &str) -> Result<String>;
    async fn table_columns(&self, table: &str) -> Result<Vec<String>>;
    async fn get_rows(&self, table: &str, column: &str) -> Result<Vec<Vec<serde_json::Value>>>;
}
