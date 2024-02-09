use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{FromRow, Sqlite, SqlitePool};
use sqlx::pool::PoolConnection;

pub struct UserModelController;

impl UserModelController {
    pub async fn get_user_by_id(server_id: &str, db_pool: &SqlitePool) -> anyhow::Result<User> {
        let mut connection = Self::acquire_connection(&db_pool).await?;

        let db_user = sqlx::query_as!(
            UserInDB,
            "SELECT * FROM users WHERE server_id=$1",
            server_id
        )
            .fetch_one(connection.as_mut())
            .await;

        return match db_user {
            Ok(db_user) => User::try_from(db_user),
            Err(e) => {
                let error_msg = format!("Failed to get user {server_id} from database: {e}");

                tracing::error!(error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    pub async fn list_users(db_pool: &SqlitePool) -> anyhow::Result<Vec<GetUser>> {
        let mut connection = Self::acquire_connection(&db_pool).await?;

        let db_users = sqlx::query_as!(GetUserFromDB, "SELECT server_id, server_list FROM users")
            .fetch_all(connection.as_mut())
            .await?;

        let users = db_users
            .into_iter()
            .map(|user| GetUser::try_from(user))
            .collect::<Result<Vec<GetUser>, _>>()?;

        Ok(users)
    }

    pub async fn add_user(data: AdminPostBody, db_pool: &SqlitePool) -> anyhow::Result<()> {
        let mut connection = Self::acquire_connection(db_pool).await?;
        let create_user = UserInDB::new(data.server_id.clone(), data.server_list, data.auth_token)?;

        let query_result = sqlx::query!(
            "INSERT INTO users (server_id, server_list, auth_token) VALUES ($1, $2, $3)",
            create_user.server_id,
            create_user.server_list,
            create_user.auth_token
        )
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

    pub async fn delete_user(data: AdminDeleteBody, db_pool: &SqlitePool) -> anyhow::Result<()> {
        let mut connection = Self::acquire_connection(db_pool).await?;

        let query_result = sqlx::query!("DELETE FROM users WHERE server_id = $1", data.server_id)
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
        db_pool: &SqlitePool,
    ) -> anyhow::Result<GetUser> {
        if data.server_list.is_some() && data.server_list.clone().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Server list cannot be empty"));
        }

        let mut connection = Self::acquire_connection(db_pool).await?;

        if data.server_list.is_some() && data.auth_token.is_some() {
            let server_list = serde_json::to_string(&data.server_list.unwrap())?;
            let auth_token = UserModelController::hash_password(data.auth_token.unwrap());

            let query_result = sqlx::query_as!(GetUserFromDB,
                "UPDATE users SET server_list = $1, auth_token = $2 WHERE server_id = $3 RETURNING server_id, server_list",
                server_list,
                auth_token,
                data.server_id
            )
                .fetch_one(connection.as_mut())
                .await?;

            return Ok(GetUser::try_from(query_result)?);
        } else if data.server_list.is_some() && data.auth_token.is_none() {
            let server_list = serde_json::to_string(&data.server_list.unwrap())?;

            let query_result = sqlx::query_as!(GetUserFromDB,
                "UPDATE users SET server_list = $1 WHERE server_id = $2 RETURNING server_id, server_list",
                server_list,
                data.server_id
            )
                .fetch_one(connection.as_mut())
                .await?;

            return Ok(GetUser::try_from(query_result)?);
        } else if data.auth_token.is_some() && data.server_list.is_none() {
            let auth_token = UserModelController::hash_password(data.auth_token.unwrap());

            let query_result = sqlx::query_as!(GetUserFromDB,
                "UPDATE users SET auth_token = $1 WHERE server_id = $2 RETURNING server_id, server_list",
                auth_token,
                data.server_id
            )
                .fetch_one(connection.as_mut())
                .await?;

            return Ok(GetUser::try_from(query_result)?);
        } else {
            return Err(anyhow::anyhow!("No data to update"));
        }
    }

    pub fn hash_password(password: String) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(password);
        hasher.update(b"lkajsdf982");
        format!("{:x}", hasher.finalize())
    }

    pub async fn acquire_connection(
        db_pool: &SqlitePool,
    ) -> anyhow::Result<PoolConnection<Sqlite>> {
        db_pool.acquire().await.map_err(|e| {
            let error_msg = format!("Failed to acquire a connection from the database pool: {e}");

            tracing::error!(error_msg);
            anyhow::anyhow!(error_msg)
        })
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

#[derive(Debug)]
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
            auth_token: UserModelController::hash_password(unhashed_auth_token),
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