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
