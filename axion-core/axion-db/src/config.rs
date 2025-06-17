// axion-db/src/config.rs
use crate::error::{DbError, DbResult};
use serde::{Deserialize, Serialize};
use sqlx::any::AnyConnectOptions;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    Postgres,
    Mysql,
    Sqlite,
}

impl Default for DatabaseType {
    fn default() -> Self {
        DatabaseType::Postgres // Default to Postgres
    }
}

impl FromStr for DatabaseType {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            "mysql" | "mariadb" => Ok(DatabaseType::Mysql),
            "sqlite" => Ok(DatabaseType::Sqlite),
            _ => Err(DbError::Config(format!("Unsupported database type: {}", s))),
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Postgres => write!(f, "PostgreSQL"),
            DatabaseType::Mysql => write!(f, "MySQL/MariaDB"),
            DatabaseType::Sqlite => write!(f, "SQLite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolOptionsConfig {
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout_seconds: Option<u64>,
    pub idle_timeout_seconds: Option<u64>,
    pub max_lifetime_seconds: Option<u64>,
    pub acquire_timeout_seconds: Option<u64>,
    pub test_before_acquire: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbConfig {
    pub db_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database_name: Option<String>,
    pub schema: Option<String>, // Default/current schema
    pub connection_string: Option<String>,
    pub pool_options: Option<PoolOptionsConfig>,
    // For SQLite, this would be the file path
    pub sqlite_path: Option<String>,
}

impl DbConfig {
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            db_type,
            ..Default::default()
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn database_name(mut self, database_name: impl Into<String>) -> Self {
        self.database_name = Some(database_name.into());
        self
    }

    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn connection_string(mut self, cs: impl Into<String>) -> Self {
        self.connection_string = Some(cs.into());
        self
    }

    pub fn pool_options(mut self, pool_opts: PoolOptionsConfig) -> Self {
        self.pool_options = Some(pool_opts);
        self
    }

    /// Builds the connection string or returns an error if essential parts are missing.
    pub fn build_connection_string(&self) -> DbResult<String> {
        if let Some(cs) = &self.connection_string {
            return Ok(cs.clone());
        }

        match self.db_type {
            DatabaseType::Postgres => Ok(format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.username
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing username for Postgres".to_string()))?,
                self.password
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing password for Postgres".to_string()))?,
                self.host
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing host for Postgres".to_string()))?,
                self.port
                    .ok_or_else(|| DbError::Config("Missing port for Postgres".to_string()))?,
                self.database_name
                    .as_deref()
                    .ok_or_else(|| DbError::Config(
                        "Missing database_name for Postgres".to_string()
                    ))?
            )),
            DatabaseType::Mysql => Ok(format!(
                "mysql://{}:{}@{}:{}/{}",
                self.username
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing username for MySQL".to_string()))?,
                self.password
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing password for MySQL".to_string()))?,
                self.host
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing host for MySQL".to_string()))?,
                self.port
                    .ok_or_else(|| DbError::Config("Missing port for MySQL".to_string()))?,
                self.database_name
                    .as_deref()
                    .ok_or_else(|| DbError::Config(
                        "Missing database_name for MySQL".to_string()
                    ))?
            )),
            DatabaseType::Sqlite => {
                let path = self
                    .sqlite_path
                    .as_deref()
                    .ok_or_else(|| DbError::Config("Missing sqlite_path for SQLite".to_string()))?;
                if path.is_empty() {
                    Ok("sqlite::memory:".to_string()) // In-memory if path is empty
                } else {
                    Ok(format!("sqlite:{}", path))
                }
            }
        }
    }

    pub fn to_sqlx_any_options(&self) -> DbResult<sqlx::any::AnyConnectOptions> {
        let cs = self.build_connection_string()?;
        AnyConnectOptions::from_str(&cs).map_err(|e| {
            DbError::Config(format!("Failed to parse connection string for sqlx: {}", e))
        })
    }
}
