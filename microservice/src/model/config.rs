use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

use crate::database_utils::DatabaseUtils;

pub struct ConfigModelController;

impl ConfigModelController {
    pub async fn delete_config(client_id: &str, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_delete")?;

        println!("query_string: {}", query_string);

        sqlx::query(query_string.as_str())
            .bind(client_id)
            .execute(connection.as_mut())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete config: {}", e))?;

        Ok(())
    }

    pub async fn add_or_update_config(
        client_id: &str,
        subscriptions: &Vec<String>,
        db_pool: SqlitePool,
    ) -> anyhow::Result<ClientConfig> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("config_add_or_update")?;

        let query_result: ClientConfig =
            sqlx::query_as::<_, ConfigInDatabase>(query_string.as_str())
                .bind(client_id)
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
