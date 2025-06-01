// axion-db/src/lib.rs
pub mod client;
pub mod config;
pub mod error;
pub mod metadata;
// pub mod introspection; // If you move introspection logic to its own module

pub mod prelude {
    pub use crate::client::DbClient;
    pub use crate::config::{DatabaseType, DbConfig, PoolOptionsConfig};
    pub use crate::error::{DbError, DbResult};
    pub use crate::metadata::*;
    // pub use crate::introspection::*; // If introspection is a separate module
}
