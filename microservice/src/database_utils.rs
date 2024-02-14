use sha2::Digest;
use sqlx::pool::PoolConnection;
use sqlx::{Sqlite, SqlitePool};

pub fn hash_password(password: String, salt: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(password);
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn acquire_connection(db_pool: &SqlitePool) -> anyhow::Result<PoolConnection<Sqlite>> {
    db_pool.acquire().await.map_err(|e| {
        let error_msg = format!("Failed to acquire a connection from the database pool: {e}");

        tracing::error!(error_msg);
        anyhow::anyhow!(error_msg)
    })
}
