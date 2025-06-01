// axion-db/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("SQLx connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Introspection error: {0}")]
    Introspection(String),

    #[error("Query execution error: {0}")]
    QueryExecution(sqlx::Error), // Keep original sqlx::Error for details

    #[error("Unsupported database type for this operation: {0}")]
    UnsupportedDbType(String),

    #[error("Type mapping error: {0}")]
    TypeMapping(String),

    #[error("Feature not enabled for database: {0}")]
    FeatureNotEnabled(String),
}

pub type DbResult<T> = Result<T, DbError>;
