use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::ctx::ctx_client::Identifier;
use crate::database_utils::acquire_connection;

pub struct ConfigModelController;

impl ConfigModelController {
    pub async fn get_config_by_identifier(
        identifier: &Identifier,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        sqlx::query_as::<_, ConfigInDatabase>("SELECT * FROM configs WHERE identifier = ?;")
            .bind(identifier.to_string())
            .fetch_optional(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to get config")?
            .map(|config| config.try_into())
            .transpose()?
            .ok_or_else(|| {
                anyhow::anyhow!("No config found for identifier: {}", identifier.to_string())
            })
    }

    pub async fn get_config_by_server_id(
        server_id: &str,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<Vec<ClientConfig>> {
        sqlx::query_as::<_, ConfigInDatabase>("SELECT * FROM configs WHERE server_id = ?;")
            .bind(server_id)
            .fetch_all(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to get config")?
            .into_iter()
            .map(|config| config.try_into())
            .collect()
    }

    pub async fn delete_config_by_server_id(
        server_id: &str,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<u64> {
        sqlx::query("DELETE FROM configs WHERE server_id = ?;")
            .bind(server_id)
            .execute(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to delete config")
            .map(|result| result.rows_affected())
    }

    pub async fn add_or_update_config(
        identifier: &Identifier,
        subscriptions: &Vec<String>,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        sqlx::query_as::<_, ConfigInDatabase>("INSERT INTO configs (identifier, server_id, client_id, subscriptions) VALUES ($1, $2, $3, $4) ON CONFLICT(identifier) DO UPDATE SET subscriptions = $4 RETURNING *;")
            .bind(identifier.to_string())
            .bind(identifier.server_id())
            .bind(identifier.client_id())
            .bind(serde_json::to_string(&subscriptions)?)
            .fetch_one(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to add or update config")?
            .try_into()
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct ConfigInDatabase {
    pub identifier: String,
    pub server_id: String,
    pub client_id: String,
    pub subscriptions: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub identifier: Identifier,
    pub subscriptions: Vec<String>,
}

impl TryFrom<ConfigInDatabase> for ClientConfig {
    type Error = anyhow::Error;

    fn try_from(config: ConfigInDatabase) -> Result<Self, Self::Error> {
        Ok(ClientConfig {
            identifier: config.identifier.parse()?,
            subscriptions: serde_json::from_str(&config.subscriptions)?,
        })
    }
}
