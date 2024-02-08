use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{pool::PoolConnection, FromRow, Sqlite, SqlitePool};

pub struct DatabaseConnection(pub PoolConnection<Sqlite>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    SqlitePool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = SqlitePool::from_ref(state);

        match pool.acquire().await {
            Ok(conn) => Ok(Self(conn)),
            Err(e) => {
                tracing::error!("Failed to acquire connection: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to acquire connection".to_string(),
                ))
            }
        }
    }
}

pub struct DatabaseHelper;

impl DatabaseHelper {
    pub async fn list_users(db_pool: SqlitePool) -> anyhow::Result<Vec<GetUser>> {
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

    pub async fn add_user(data: AdminPostBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = Self::acquire_connection(&db_pool).await?;
        let create_user = CreateUser::new(data.server_id, data.server_list, data.auth_token)?;

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
                tracing::error!("User already exists");
                return Err(anyhow::anyhow!("User already exists"));
            }

            tracing::error!("Failed to add user: {}", e);
            return Err(anyhow::anyhow!("Failed to add user"));
        }

        Ok(())
    }

    pub async fn delete_user(data: AdminDeleteBody, db_pool: SqlitePool) -> anyhow::Result<()> {
        let mut connection = Self::acquire_connection(&db_pool).await?;

        let query_result = sqlx::query!("DELETE FROM users WHERE server_id = $1", data.server_id)
            .execute(connection.as_mut())
            .await;

        if let Err(e) = query_result {
            tracing::error!("Failed to delete user: {}", e);
            return Err(anyhow::anyhow!("Failed to delete user"));
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

        let mut connection = Self::acquire_connection(&db_pool).await?;

        if data.server_list.is_some() && data.auth_token.is_some() {
            let server_list = serde_json::to_string(&data.server_list.unwrap())?;
            let auth_token = DatabaseHelper::hash_password(data.auth_token.unwrap());

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
            let auth_token = DatabaseHelper::hash_password(data.auth_token.unwrap());

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
            tracing::error!("Failed to acquire a connection: {}", e);
            anyhow::anyhow!("Failed to acquire a connection")
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
pub struct CreateUser {
    pub server_id: String,
    pub server_list: String,
    pub auth_token: String,
}

impl CreateUser {
    pub fn new(
        server_id: String,
        server_list: Vec<String>,
        unhashed_auth_token: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            server_id,
            server_list: serde_json::to_string(&server_list)?,
            auth_token: DatabaseHelper::hash_password(unhashed_auth_token),
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
