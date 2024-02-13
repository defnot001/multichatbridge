use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::database_utils::{acquire_connection, hash_password};

pub struct UserModelController;

impl UserModelController {
    pub async fn get_user_by_id(server_id: &str, db_pool: &SqlitePool) -> anyhow::Result<User> {
        sqlx::query_as::<_, UserInDB>("SELECT * FROM users WHERE server_id = ?;")
            .bind(server_id)
            .fetch_optional(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to get user: {e}")?
            .map(|user| user.try_into())
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    pub async fn list_users(db_pool: &SqlitePool) -> anyhow::Result<Vec<UserNoToken>> {
        sqlx::query_as::<_, UserNoTokenInDB>("SELECT server_id, server_list FROM users;")
            .fetch_all(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to get users from database")?
            .into_iter()
            .map(|user| user.try_into())
            .collect()
    }

    pub async fn add_user(
        mut data: AdminPostBody,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<UserNoToken> {
        add_if_not_contains(&mut data.server_list, "discord".to_string());

        sqlx::query_as::<_, UserNoTokenInDB>(
            "INSERT INTO users (server_id, server_list, auth_token) VALUES (?, ?, ?) RETURNING *;",
        )
        .bind(data.server_id)
        .bind(serde_json::to_string(&data.server_list).context("Failed to serialize serverlist")?)
        .bind(data.auth_token)
        .fetch_one(acquire_connection(db_pool).await?.as_mut())
        .await
        .context("Failed to add user")?
        .try_into()
    }

    pub async fn delete_user(
        data: AdminDeleteBody,
        db_pool: &SqlitePool,
    ) -> anyhow::Result<UserNoToken> {
        sqlx::query_as::<_, UserNoTokenInDB>("DELETE FROM users WHERE server_id = ? RETURNING *;")
            .bind(data.server_id)
            .fetch_optional(acquire_connection(db_pool).await?.as_mut())
            .await
            .context("Failed to delete user")?
            .map(|user| user.try_into())
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    pub async fn update_user(
        data: AdminUpdateBody,
        db_pool: SqlitePool,
    ) -> anyhow::Result<UserNoToken> {
        if let Some(mut server_list) = data.server_list.clone() {
            if server_list.is_empty() {
                return Err(anyhow::anyhow!("Server list cannot be empty"));
            }

            add_if_not_contains(server_list.as_mut(), "discord".to_string());
        }

        let stringified_server_list =
            if let Ok(server_list) = serde_json::to_string(&data.server_list) {
                Some(server_list)
            } else {
                None
            };

        let hashed_token = if let Some(token) = data.auth_token {
            Some(hash_password(token))
        } else {
            None
        };

        let sql_query = r#"
            UPDATE users SET
                server_list = CASE WHEN $1 IS NOT NULL THEN $1 ELSE server_list END,
                auth_token = CASE WHEN $2 IS NOT NULL THEN $2 ELSE auth_token END
            WHERE server_id = $3
            RETURNING server_id, server_list;
        "#;

        return sqlx::query_as::<_, UserNoTokenInDB>(sql_query)
            .bind(stringified_server_list)
            .bind(hashed_token)
            .bind(data.server_id)
            .fetch_optional(acquire_connection(&db_pool).await?.as_mut())
            .await
            .context("Failed to update user")?
            .map(|user| user.try_into())
            .context("Failed to deserialize user")?;
    }
}

fn add_if_not_contains<T: PartialEq>(list: &mut Vec<T>, item: T) {
    if !list.contains(&item) {
        list.push(item);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminPostBody {
    pub server_id: String,
    pub server_list: Vec<String>,
    pub auth_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminDeleteBody {
    pub server_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminUpdateBody {
    pub server_id: String,
    pub server_list: Option<Vec<String>>,
    pub auth_token: Option<String>,
}

#[derive(Debug)]
pub struct User {
    pub server_id: String,
    pub server_list: Vec<String>,
    pub hashed_auth_token: String,
}

impl TryFrom<UserInDB> for User {
    type Error = anyhow::Error;

    fn try_from(user: UserInDB) -> Result<Self, Self::Error> {
        Ok(Self {
            server_id: user.server_id,
            server_list: serde_json::from_str(&user.server_list)?,
            hashed_auth_token: user.auth_token,
        })
    }
}

#[derive(Debug, FromRow)]
pub struct UserInDB {
    pub server_id: String,
    pub server_list: String,
    pub auth_token: String,
}

impl UserInDB {
    pub fn new(
        server_id: String,
        server_list: Vec<String>,
        unhashed_auth_token: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            server_id,
            server_list: serde_json::to_string(&server_list)?,
            auth_token: hash_password(unhashed_auth_token),
        })
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct UserNoTokenInDB {
    pub server_id: String,
    pub server_list: String,
}

#[derive(Debug, Serialize)]
pub struct UserNoToken {
    pub server_id: String,
    pub server_list: Vec<String>,
}

impl TryFrom<UserNoTokenInDB> for UserNoToken {
    type Error = anyhow::Error;

    fn try_from(user: UserNoTokenInDB) -> Result<Self, Self::Error> {
        Ok(Self {
            server_id: user.server_id,
            server_list: serde_json::from_str(&user.server_list)?,
        })
    }
}
