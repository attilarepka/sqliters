use anyhow::Result;
use serde_json::{json, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
    AssertSqlSafe, Column, Row, TypeInfo,
};

use crate::database::{Database, SqliteDb};

impl SqliteDb {
    pub async fn connect(path: &str, create_if_missing: bool) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(create_if_missing);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn memory() -> Result<Self> {
        let options = SqliteConnectOptions::new().in_memory(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }
}

impl Database for SqliteDb {
    async fn tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&self.pool)
            .await?;

        let table_names = rows
            .into_iter()
            .map(|row: SqliteRow| row.get::<String, &str>("name"))
            .collect::<Vec<String>>();

        Ok(table_names)
    }

    async fn schema(&self, table: &str) -> Result<String> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT sql
            FROM sqlite_schema
            WHERE tbl_name = ?1
              AND sql IS NOT NULL
            ORDER BY type = 'table' DESC, type, name
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.join(";\n"))
    }

    async fn columns(&self, table: &str) -> Result<Vec<String>> {
        let query = format!("PRAGMA table_info({table})");

        let rows = sqlx::query(AssertSqlSafe(query.as_str()))
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.try_get::<String, _>("name"))
            .collect::<Result<_, _>>()?)
    }

    async fn rows(&self, column: &str, table: &str) -> Result<Vec<Vec<Value>>> {
        let query = format!("SELECT {column} FROM {table}");

        let result: Vec<_> = sqlx::query(AssertSqlSafe(query.as_str()))
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| {
                        let ordinal = column.ordinal();
                        let type_name = column.type_info().name();
                        match type_name {
                            "NULL" => json!("null".to_string()),
                            "INTEGER" => json!(row.get::<i64, _>(ordinal).to_string()),
                            "REAL" => json!(row.get::<f64, _>(ordinal).to_string()),
                            "TEXT" | "DATETIME" => {
                                json!(row.get::<String, _>(ordinal).to_string())
                            }
                            "BLOB" => {
                                json!(hex::encode(row.get::<Vec<u8>, _>(ordinal)).to_string())
                            }
                            _ => {
                                panic!("not supported type: {type_name}");
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_db() -> SqliteDb {
        SqliteDb::memory().await.unwrap()
    }

    #[tokio::test]
    async fn test_db_init() {
        let db = test_db().await;
        assert_eq!(db.tables().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_db_table_schema_unhappy() {
        let db = test_db().await;
        assert!(db.schema("users").await.is_ok());
    }
}
