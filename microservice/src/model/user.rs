use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::database_utils::DatabaseUtils;

pub struct UserModelController;

impl UserModelController {
    pub async fn get_user_by_id(server_id: &str, db_pool: SqlitePool) -> anyhow::Result<User> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("user_get_by_id")?;

        let db_user = sqlx::query_as::<_, UserInDB>(query_string.as_str())
            .bind(server_id)
            .fetch_optional(connection.as_mut())
            .await?;

        match db_user {
            Some(db_user) => User::try_from(db_user),
            None => Err(anyhow::anyhow!(
                "User {server_id} does not exist in the database"
            )),
        }
    }

    pub async fn list_users(db_pool: SqlitePool) -> anyhow::Result<Vec<GetUser>> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("user_get_all")?;

        let query_result = sqlx::query_as::<_, GetUserFromDB>(query_string.as_str())
            .fetch_all(connection.as_mut())
            .await;

        let users = match query_result {
            Ok(users) => users
                .into_iter()
                .map(GetUser::try_from)
                .collect::<Result<Vec<GetUser>, _>>()?,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get users from database: {e}"));
            }
        };

        Ok(users)
    }

    pub async fn add_user(mut data: AdminPostBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        if !data.server_list.contains(&"discord".to_string()) {
            data.server_list.push("discord".to_string());
        }

        let create_user = UserInDB::new(data.server_id.clone(), data.server_list, data.auth_token)?;

        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("user_add")?;

        let query_result = sqlx::query(query_string.as_str())
            .bind(create_user.server_id)
            .bind(create_user.server_list)
            .bind(create_user.auth_token)
            .execute(connection.as_mut())
            .await;

        if let Err(e) = query_result {
            if e.to_string().contains("UNIQUE constraint failed") {
                return Err(anyhow::anyhow!("User {} already exists", data.server_id));
            }

            return Err(anyhow::anyhow!("Failed to add user: {}", e));
        }

        Ok(())
    }

    pub async fn delete_user(data: AdminDeleteBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let query_string = DatabaseUtils::query_from_file("user_delete")?;

        let query_result = sqlx::query(query_string.as_str())
            .bind(&data.server_id)
            .execute(connection.as_mut())
            .await;

        if let Err(e) = query_result {
            return Err(anyhow::anyhow!(
                "Failed to delete user {}: {e}",
                data.server_id
            ));
        }

        Ok(())
    }

    pub async fn update_user(
        mut data: AdminUpdateBody,
        db_pool: SqlitePool,
    ) -> anyhow::Result<GetUser> {
        if data.server_list.is_some() && data.server_list.clone().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Server list cannot be empty"));
        }

        if let Some(server_list) = &data.server_list {
            if !server_list.contains(&"discord".to_string()) {
                data.server_list
                    .as_mut()
                    .unwrap()
                    .push("discord".to_string());
            }
        }

        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        if data.server_list.is_some() && data.auth_token.is_some() {
            let stringified_server_list = serde_json::to_string(&data.server_list.unwrap())?;
            let hashed_auth_token = DatabaseUtils::hash_password(data.auth_token.unwrap());

            let query_string = DatabaseUtils::query_from_file("user_update_serverlist_authtoken")?;

            let query_result = sqlx::query_as::<_, GetUserFromDB>(query_string.as_str())
                .bind(stringified_server_list)
                .bind(hashed_auth_token)
                .bind(data.server_id)
                .fetch_one(connection.as_mut())
                .await;

            match query_result {
                Ok(query_result) => GetUser::try_from(query_result),
                Err(e) => Err(anyhow::anyhow!("Failed to update user: {}", e)),
            }
        } else if data.server_list.is_some() && data.auth_token.is_none() {
            let stringified_server_list = serde_json::to_string(&data.server_list.unwrap())?;

            let query_string = DatabaseUtils::query_from_file("user_update_serverlist")?;

            let query_result = sqlx::query_as::<_, GetUserFromDB>(query_string.as_str())
                .bind(stringified_server_list)
                .bind(data.server_id)
                .fetch_one(connection.as_mut())
                .await;

            match query_result {
                Ok(query_result) => GetUser::try_from(query_result),
                Err(e) => Err(anyhow::anyhow!("Failed to update user: {}", e)),
            }
        } else if data.auth_token.is_some() && data.server_list.is_none() {
            let hashed_auth_token = DatabaseUtils::hash_password(data.auth_token.unwrap());

            let query_string = DatabaseUtils::query_from_file("user_update_authtoken")?;

            let query_result = sqlx::query_as::<_, GetUserFromDB>(query_string.as_str())
                .bind(hashed_auth_token)
                .fetch_one(connection.as_mut())
                .await;

            match query_result {
                Ok(query_result) => GetUser::try_from(query_result),
                Err(e) => Err(anyhow::anyhow!("Failed to update user: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("No data to update"))
        }
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
            auth_token: DatabaseUtils::hash_password(unhashed_auth_token),
        })
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct GetUserFromDB {
    pub server_id: String,
    pub server_list: String,
}

#[derive(Debug, Serialize)]
pub struct GetUser {
    pub server_id: String,
    pub server_list: Vec<String>,
}

impl TryFrom<GetUserFromDB> for GetUser {
    type Error = anyhow::Error;

    fn try_from(user: GetUserFromDB) -> Result<Self, Self::Error> {
        Ok(Self {
            server_id: user.server_id,
            server_list: serde_json::from_str(&user.server_list)?,
        })
    }
}
