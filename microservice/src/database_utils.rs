use std::path::PathBuf;

use sha2::Digest;
use sqlx::pool::PoolConnection;
use sqlx::{Sqlite, SqlitePool};

pub struct DatabaseUtils {
    db_pool: SqlitePool,
}

impl DatabaseUtils {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    pub fn query_from_file(file_name: &str) -> anyhow::Result<String> {
        let file_path = PathBuf::from("database")
            .join("sql")
            .join(format!("{}.sql", file_name));

        std::fs::read_to_string(file_path).map_err(|e| {
            let error_msg = format!("Failed to read the SQL query file: {e}");

            tracing::error!(error_msg);
            anyhow::anyhow!(error_msg)
        })
    }

    pub fn hash_password(password: String) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(password);
        hasher.update(b"lkajsdf982");
        format!("{:x}", hasher.finalize())
    }

    pub async fn connection(&self) -> anyhow::Result<PoolConnection<Sqlite>> {
        self.db_pool.acquire().await.map_err(|e| {
            let error_msg = format!("Failed to acquire a connection from the database pool: {e}");

            tracing::error!(error_msg);
            anyhow::anyhow!(error_msg)
        })
    }
}
