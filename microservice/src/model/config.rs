use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::ctx::ctx_client::Identifier;
use crate::database_utils::DatabaseUtils;

pub struct ConfigModelController;

impl ConfigModelController {
    pub async fn get_config_by_identifier(
        identifier: Identifier,
        db_pool: SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_get_by_identifier")?;

        let query_result = sqlx::query_as::<_, ConfigInDatabase>(query_string.as_str())
            .bind(identifier.to_string())
            .fetch_optional(connection.as_mut())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get config: {}", e))?;

        let query_result = match query_result {
            Some(config) => config.try_into()?,
            None => {
                return Err(anyhow::anyhow!(
                    "No config found for identifier: {}",
                    identifier.to_string()
                ))
            }
        };

        Ok(query_result)
    }

    pub async fn get_config_by_server_id(
        server_id: &str,
        db_pool: SqlitePool,
    ) -> anyhow::Result<Vec<ClientConfig>> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_get_by_server_id")?;

        let query_result: Vec<ConfigInDatabase> =
            sqlx::query_as::<_, ConfigInDatabase>(query_string.as_str())
                .bind(server_id)
                .fetch_all(connection.as_mut())
                .await?;

        let query_result: Vec<ClientConfig> = query_result
            .into_iter()
            .map(|config| config.try_into())
            .collect::<Result<Vec<ClientConfig>, anyhow::Error>>()?;

        if query_result.is_empty() {
            return Err(anyhow::anyhow!(
                "No configs found for server_id: {}",
                server_id
            ));
        }

        Ok(query_result)
    }

    pub async fn delete_config(server_id: &str, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_delete")?;

        sqlx::query(query_string.as_str())
            .bind(server_id)
            .execute(connection.as_mut())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete config: {}", e))?;

        Ok(())
    }

    pub async fn add_or_update_config(
        identifier: &Identifier,
        subscriptions: &Vec<String>,
        db_pool: SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_add_or_update")?;

        let query_result: ClientConfig =
            sqlx::query_as::<_, ConfigInDatabase>(query_string.as_str())
                .bind(identifier.to_string())
                .bind(identifier.server_id())
                .bind(identifier.client_id())
                .bind(serde_json::to_string(&subscriptions)?)
                .fetch_one(connection.as_mut())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add or update config: {}", e))?
                .try_into()?;

        Ok(query_result)
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
    pub server_id: String,
    pub client_id: String,
    pub subscriptions: Vec<String>,
}

impl TryFrom<ConfigInDatabase> for ClientConfig {
    type Error = anyhow::Error;

    fn try_from(config: ConfigInDatabase) -> Result<Self, Self::Error> {
        Ok(ClientConfig {
            identifier: config.identifier.parse()?,
            server_id: config.server_id,
            client_id: config.client_id,
            subscriptions: serde_json::from_str(&config.subscriptions)?,
        })
    }
}
