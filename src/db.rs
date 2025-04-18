#![allow(dead_code)]

use anyhow::Result;
use serde_json::{json, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
    Column, QueryBuilder, Row, SqlitePool, TypeInfo,
};

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    pub async fn new() -> Result<Sqlite> {
        let sqlite_opts = SqliteConnectOptions::new().in_memory(true);
        Ok(Sqlite {
            pool: SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(sqlite_opts)
                .await?,
        })
    }

    pub async fn from(path: &str, create_if_missing: bool) -> Result<Sqlite> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(create_if_missing);

        Ok(Sqlite {
            pool: SqlitePool::connect_with(options).await?,
        })
    }

    pub async fn tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&self.pool)
            .await?;

        let table_names = rows
            .into_iter()
            .map(|row: SqliteRow| row.get::<String, &str>("name"))
            .collect::<Vec<String>>();

        Ok(table_names)
    }

    pub async fn schema(&self) -> Result<String> {
        let rows = sqlx::query("SELECT sql FROM sqlite_schema")
            .fetch_all(&self.pool)
            .await?;

        let mut result = String::new();
        for row in rows {
            let schema = row.get::<String, &str>("sql");
            result.push_str(schema.as_str());
        }

        Ok(result)
    }

    pub async fn table_schema(&self, table: &str) -> Result<String> {
        let query = format!("SELECT sql FROM sqlite_schema WHERE name='{table}'");
        let rows = sqlx::query(query.as_str()).fetch_all(&self.pool).await?;

        let mut result = String::new();
        for row in rows {
            let schema = row.get::<String, &str>("sql");
            result.push_str(schema.as_str());
        }

        Ok(result)
    }

    pub async fn table_columns(&self, table: &str) -> Result<Vec<String>> {
        let query = format!("PRAGMA table_info({table})");
        let rows = sqlx::query(query.as_str()).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row: SqliteRow| row.get::<String, &str>("name"))
            .collect::<Vec<String>>())
    }

    pub async fn insert_rows(&self, table: &str, column: &str, rows: &Vec<&str>) -> Result<u64> {
        let query = format!("INSERT INTO {table} ({column}) ");
        let mut query_builder = QueryBuilder::new(query.as_str());

        query_builder.push_values(rows, |mut query, row| {
            query.push_bind(row);
        });

        let query = query_builder.build();

        Ok(query.execute(&self.pool).await?.rows_affected())
    }

    pub async fn remove_row(&self, row: &str, table: &str) -> Result<u64> {
        let query = format!("DELETE FROM {table} WHERE {row}");
        let result = sqlx::query(query.as_str()).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn remove_column(&self, column: &str, table: &str) -> Result<u64> {
        let query = format!("ALTER TABLE {table} DROP COLUMN {column}");
        let result = sqlx::query(query.as_str()).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn create_table(&self, name: &str, query: &str) -> Result<u64> {
        let query = format!("CREATE TABLE {name} ({query})");

        let result = sqlx::query(query.as_str()).execute(&self.pool).await?;

        Ok(result.rows_affected())
    }

    pub async fn remove_table(&self, table: &str) -> Result<u64> {
        let query = format!("DROP TABLE {table}");
        let result = sqlx::query(query.as_str()).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn get_column_type(&self, column: &str, table: &str) -> Result<String> {
        let query = format!("SELECT type FROM pragma_table_info('{table}') WHERE name='{column}'");
        let rows = sqlx::query(query.as_str()).fetch_all(&self.pool).await?;
        let column_type = rows
            .into_iter()
            .map(|row: SqliteRow| row.get::<String, &str>("type"))
            .collect::<Vec<String>>()[0]
            .clone();
        Ok(column_type)
    }

    pub async fn get_rows(&self, column: &str, table: &str) -> Result<Vec<Vec<Value>>> {
        let query = format!("SELECT {column} FROM {table};");

        let result: Vec<_> = sqlx::query(&query)
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

    #[tokio::test]
    async fn test_db_init() {
        let db = Sqlite::new().await.unwrap();
        assert_eq!(db.tables().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_db_schema_unhappy() {
        let db = Sqlite::new().await.unwrap();
        assert!(db.schema().await.is_ok());
    }

    #[tokio::test]
    async fn test_db_table_schema_unhappy() {
        let db = Sqlite::new().await.unwrap();
        assert!(db.table_schema("users").await.is_ok());
    }

    #[tokio::test]
    async fn test_db_insert_row_unhappy() {
        let db = Sqlite::new().await.unwrap();
        assert!(db
            .insert_rows("users", "id", &vec!["1", "2", "3"])
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_db_remove_row_unhappy() {
        let db = Sqlite::new().await.unwrap();
        assert!(db.remove_row("1, 2, 3", "users").await.is_err());
    }

    #[tokio::test]
    async fn test_db_remove_column_unhappy() {
        let db = Sqlite::new().await.unwrap();
        assert!(db.remove_column("id", "users").await.is_err());
    }

    #[tokio::test]
    async fn test_db_create_table() {
        const TABLE_NAME: &str = "users";
        let db = Sqlite::new().await.unwrap();
        const COLUMN_NAME: &str = "id";
        const COLUMN_TYPE: &str = "INTEGER";
        assert!(db
            .create_table(TABLE_NAME, format!("{COLUMN_NAME} {COLUMN_TYPE}").as_str())
            .await
            .is_ok());

        assert_eq!(
            db.schema().await.unwrap(),
            format!("CREATE TABLE {TABLE_NAME} ({COLUMN_NAME} {COLUMN_TYPE})")
        );

        assert_eq!(db.tables().await.unwrap(), vec![TABLE_NAME]);
        assert_eq!(
            db.get_column_type(COLUMN_NAME, TABLE_NAME).await.unwrap(),
            "INTEGER"
        );

        assert!(db
            .insert_rows(TABLE_NAME, COLUMN_NAME, &vec!["1", "2", "3"])
            .await
            .is_ok());

        assert_eq!(
            db.get_rows(COLUMN_NAME, TABLE_NAME).await.unwrap(),
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()]
            ]
        );

        assert_eq!(db.remove_table(TABLE_NAME).await.unwrap(), 3);
    }
}
