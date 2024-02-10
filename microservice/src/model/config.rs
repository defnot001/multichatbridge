use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

use crate::database_utils::DatabaseUtils;

pub struct ConfigModelController;

impl ConfigModelController {
    // pub async fn list_configs(db_pool: SqlitePool) -> anyhow::Result<Vec<ClientConfig>> {
    //     let mut connection = DatabaseUtils::new(db_pool).connection().await?;
    //
    //     let db_config = sqlx::query_as::<_, ConfigInDatabase>("SELECT * FROM configs")
    //         .fetch_all(connection.as_mut())
    //         .await;
    //
    //     match db_config {
    //         Ok(db_config) => {
    //             let config = db_config
    //                 .into_iter()
    //                 .map(ClientConfig::try_from)
    //                 .collect::<Result<Vec<ClientConfig>, _>>()?;
    //
    //             Ok(config)
    //         }
    //         Err(e) => {
    //             let error_msg = format!("Failed to get configs from database: {e}");
    //
    //             tracing::error!(error_msg);
    //             Err(anyhow::anyhow!(error_msg))
    //         }
    //     }
    // }

    // pub async fn get_config(client_id: &str, db_pool: SqlitePool) -> anyhow::Result<ClientConfig> {
    //     let mut connection = DatabaseUtils::new(db_pool).connection().await?;
    //
    //     let db_config =
    //         sqlx::query_as::<_, ConfigInDatabase>("SELECT * FROM configs WHERE client_id=?")
    //             .bind(client_id)
    //             .fetch_one(connection.as_mut())
    //             .await;
    //
    //     match db_config {
    //         Ok(db_config) => ClientConfig::try_from(db_config),
    //         Err(e) => {
    //             let error_msg = format!("Failed to get config {client_id} from database: {e}");
    //
    //             tracing::error!(error_msg);
    //             Err(anyhow::anyhow!(error_msg))
    //         }
    //     }
    // }

    // pub async fn add_config(
    //     client_id: &str,
    //     subscriptions: Vec<String>,
    //     db_pool: SqlitePool,
    // ) -> anyhow::Result<()> {
    //     let mut connection = DatabaseUtils::new(db_pool).connection().await?;
    //
    //     let query_result =
    //         sqlx::query("INSERT INTO configs (client_id, subscriptions) VALUES ($1, $2)")
    //             .bind(client_id)
    //             .bind(serde_json::to_string(&subscriptions)?)
    //             .execute(connection.as_mut())
    //             .await;
    //
    //     if let Err(e) = query_result {
    //         if e.to_string().contains("UNIQUE constraint failed") {
    //             let error_msg = format!("Config {client_id} already exists");
    //
    //             tracing::error!(error_msg);
    //             return Err(anyhow::anyhow!(error_msg));
    //         }
    //     }
    //
    //     Ok(())
    // }

    // pub async fn update_config(
    //     client_id: &str,
    //     subscriptions: Vec<String>,
    //     db_pool: SqlitePool,
    // ) -> anyhow::Result<()> {
    //     let mut connection = DatabaseUtils::new(db_pool).connection().await?;
    //
    //     let query_result = sqlx::query("UPDATE configs SET subscriptions=$1 WHERE client_id=$2")
    //         .bind(serde_json::to_string(&subscriptions)?)
    //         .bind(client_id)
    //         .execute(connection.as_mut())
    //         .await;
    //
    //     if let Err(e) = query_result {
    //         let error_msg = format!("Failed to update config: {}", e);
    //
    //         tracing::error!(error_msg);
    //         return Err(anyhow::anyhow!(error_msg));
    //     }
    //
    //     Ok(())
    // }

    // pub async fn delete_config(client_id: &str, db_pool: SqlitePool) -> anyhow::Result<()> {
    //     let mut connection = DatabaseUtils::new(db_pool).connection().await?;
    //
    //     let query_result = sqlx::query("DELETE FROM configs WHERE client_id=$1")
    //         .bind(client_id)
    //         .execute(connection.as_mut())
    //         .await;
    //
    //     if let Err(e) = query_result {
    //         let error_msg = format!("Failed to delete config: {}", e);
    //
    //         tracing::error!(error_msg);
    //         return Err(anyhow::anyhow!(error_msg));
    //     }
    //
    //     Ok(())
    // }

    pub async fn add_or_update_config(
        client_id: &str,
        subscriptions: &Vec<String>,
        db_pool: SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        let query_result = sqlx::query_as::<_, ConfigInDatabase>("INSERT INTO configs (client_id, subscriptions) VALUES ($1, $2) ON CONFLICT(client_id) DO UPDATE SET subscriptions=$2 RETURNING *")
            .bind(client_id)
            .bind(serde_json::to_string(&subscriptions)?)
            .fetch_one(connection.as_mut())
            .await;

        if let Err(e) = query_result {
            let error_msg = format!("Failed to add or update config: {}", e);

            tracing::error!(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        ClientConfig::try_from(query_result?)
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct ConfigInDatabase {
    pub client_id: String,
    pub subscriptions: String,
}

#[derive(Debug)]
pub struct ClientConfig {
    pub client_id: String,
    pub subscriptions: Vec<String>,
}

impl TryFrom<ConfigInDatabase> for ClientConfig {
    type Error = anyhow::Error;

    fn try_from(config: ConfigInDatabase) -> Result<Self, Self::Error> {
        Ok(ClientConfig {
            client_id: config.client_id,
            subscriptions: serde_json::from_str(config.subscriptions.as_str())?,
        })
    }
}
