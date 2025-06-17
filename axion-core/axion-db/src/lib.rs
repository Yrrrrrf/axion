// axion-db/src/lib.rs
pub mod client;
pub mod config;
pub mod error;
pub mod introspection;
pub mod metadata;
pub mod types;

pub mod prelude {
    pub use crate::client::DbClient;
    pub use crate::config::{DatabaseType, DbConfig, PoolOptionsConfig};
    pub use crate::error::{DbError, DbResult};
    pub use crate::introspection::{Introspector, postgres::PostgresIntrospector};
    pub use crate::metadata::*;
}
