use std::str::FromStr;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_with::serde_derive::Serialize;

/// ClientID represents a unique identifier for a client.
///
/// It is composed of a server ID and a client ID.
///
/// The server ID is used to identify the server that the client is connected to. The client ID is used to identify the client.
///
/// The format of the client ID is `server_id:client_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub server_id: String,
    pub client_id: String,
}

impl Identifier {
    /// Creates a new [Identifier] from a server_id and a client_id.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let id = Identifier::new("kiwitech", "smp");
    /// ```
    pub fn new(server_id: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            server_id: server_id.into(),
            client_id: client_id.into(),
        }
    }

    /// Returns the server ID.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Tries to create a new [Identifier] from a string representation.
    ///
    /// The format of the client ID is `server_id:client_id`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let id = Identifier::try_from_string("kiwitech:smp").unwrap();
    /// assert_eq!(id.server_id(), "kiwitech");
    /// assert_eq!(id.client_id(), "smp");
    /// ```
    pub fn try_from_string(s: impl Into<String>) -> Result<Self, anyhow::Error> {
        let s = s.into();
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid client ID format"));
        }
        Ok(Self::new(parts[0].to_string(), parts[1].to_string()))
    }
}

impl From<(String, String)> for Identifier {
    fn from((namespace, id): (String, String)) -> Self {
        Self::new(namespace, id)
    }
}

impl From<(&str, &str)> for Identifier {
    fn from((namespace, id): (&str, &str)) -> Self {
        Self::new(namespace, id)
    }
}

impl TryFrom<String> for Identifier {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from_string(s)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(s)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.server_id, self.client_id)
    }
}

impl FromStr for Identifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_string(s)
    }
}

/// [ClientCtx] represents the context of a client.
///
/// It contains the [Identifier] of the client.
#[derive(Debug, Clone)]
pub struct ClientCtx {
    pub identifier: Identifier,
}

impl ClientCtx {
    /// Creates a new [ClientCtx] from anything that implements Into<[Identifier]>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let ctx = ClientCtx::new("kiwitech:smp");
    /// let ctx2 = ClientCtx::new(Identifier::new("kiwitech", "smp"));
    /// ```
    pub fn new(identifier: impl Into<Identifier>) -> Self {
        Self {
            identifier: identifier.into(),
        }
    }

    /// Returns a reference to the [Identifier] of the client.
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ClientCtx {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let client_id_str = parts
            .headers
            .get("X-Client-ID")
            .and_then(|header| header.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing X-Client-ID header"))?;

        let client_id = Identifier::try_from_string(client_id_str);

        match client_id {
            Ok(client_id) => Ok(Self::new(client_id)),
            Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid X-Client-ID header")),
        }
    }
}
