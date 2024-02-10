use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::database_utils::DatabaseUtils;

pub struct UserModelController;

impl UserModelController {
    pub async fn get_user_by_id(server_id: &str, db_pool: SqlitePool) -> anyhow::Result<User> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        let db_user = sqlx::query_as::<_, UserInDB>("SELECT * FROM users WHERE server_id=?")
            .bind(server_id)
            .fetch_one(connection.as_mut())
            .await;

        match db_user {
            Ok(db_user) => User::try_from(db_user),
            Err(e) => {
                let error_msg = format!("Failed to get user {server_id} from database: {e}");

                tracing::error!(error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    pub async fn list_users(db_pool: SqlitePool) -> anyhow::Result<Vec<GetUser>> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        let db_users =
            sqlx::query_as::<_, GetUserFromDB>("SELECT server_id, server_list FROM users")
                .fetch_all(connection.as_mut())
                .await?;

        let users = db_users
            .into_iter()
            .map(GetUser::try_from)
            .collect::<Result<Vec<GetUser>, _>>()?;

        Ok(users)
    }

    pub async fn add_user(data: AdminPostBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;
        let create_user = UserInDB::new(data.server_id.clone(), data.server_list, data.auth_token)?;

        let query_result = sqlx::query(
            "INSERT INTO users (server_id, server_list, auth_token) VALUES ($1, $2, $3)",
        )
        .bind(create_user.server_id)
        .bind(create_user.server_list)
        .bind(create_user.auth_token)
        .execute(connection.as_mut())
        .await;

        if let Err(e) = query_result {
            if e.to_string().contains("UNIQUE constraint failed") {
                let error_msg = format!("User {} already exists", data.server_id);

                tracing::error!(error_msg);
                return Err(anyhow::anyhow!(error_msg));
            }
            let error_msg = format!("Failed to add user: {}", e);

            tracing::error!(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        Ok(())
    }

    pub async fn delete_user(data: AdminDeleteBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        let query_result = sqlx::query("DELETE FROM users WHERE server_id = ?")
            .bind(&data.server_id)
            .execute(connection.as_mut())
            .await;

        if let Err(e) = query_result {
            let error_msg = format!("Failed to delete user {}: {e}", data.server_id);

            tracing::error!(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        Ok(())
    }

    pub async fn update_user(
        data: AdminUpdateBody,
        db_pool: SqlitePool,
    ) -> anyhow::Result<GetUser> {
        if data.server_list.is_some() && data.server_list.clone().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Server list cannot be empty"));
        }

        let mut connection = DatabaseUtils::new(db_pool).connection().await?;

        if data.server_list.is_some() && data.auth_token.is_some() {
            let stringified_server_list = serde_json::to_string(&data.server_list.unwrap())?;
            let hashed_auth_token = DatabaseUtils::hash_password(data.auth_token.unwrap());

            let query_result = sqlx::query_as::<_, GetUserFromDB>("UPDATE users SET server_list = $1, auth_token = $2 WHERE server_id = $3 RETURNING server_id, server_list")
                .bind(stringified_server_list)
                .bind(hashed_auth_token)
                .bind(data.server_id)
                .fetch_one(connection.as_mut())
                .await?;

            GetUser::try_from(query_result)
        } else if data.server_list.is_some() && data.auth_token.is_none() {
            let stringified_server_list = serde_json::to_string(&data.server_list.unwrap())?;

            let query_result = sqlx::query_as::<_, GetUserFromDB>("UPDATE users SET server_list = $1 WHERE server_id = $2 RETURNING server_id, server_list")
                .bind(stringified_server_list)
                .bind(data.server_id)
                .fetch_one(connection.as_mut())
                .await?;

            GetUser::try_from(query_result)
        } else if data.auth_token.is_some() && data.server_list.is_none() {
            let hashed_auth_token = DatabaseUtils::hash_password(data.auth_token.unwrap());

            let query_result = sqlx::query_as::<_, GetUserFromDB>("UPDATE users SET auth_token = $1 WHERE server_id = $2 RETURNING server_id, server_list")
                .bind(hashed_auth_token)
                .fetch_one(connection.as_mut())
                .await?;

            GetUser::try_from(query_result)
        } else {
            Err(anyhow::anyhow!("No data to update"))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminPostBody {
    pub server_id: String,
    pub server_list: Vec<String>,
    pub auth_token: String,
}

#[derive(Debug, Clone, Deserialize)]
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
