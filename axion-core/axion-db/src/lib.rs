#![allow(unused)]

// axion-db/src/lib.rs

// These modules contain the internal implementation details.
// They are `pub` so they can be used by other modules within this crate,
// but they will NOT be part of the public `prelude`.
pub mod client;
pub mod config;
pub mod error;
pub mod introspection;
pub mod manager;
pub mod metadata;
pub mod types;

/// The public-facing prelude for the `axion-db` crate.
/// This is the ONLY part that the `axion` crate should interact with.
/// It exposes the high-level manager and the data structures it returns.
pub mod prelude {
    // The primary entry point for using this crate.
    pub use crate::manager::ModelManager;

    // The configuration struct needed to create a ModelManager.
    pub use crate::config::{DatabaseType, DbConfig, PoolOptionsConfig};

    // The error types that can be returned.
    pub use crate::error::{DbError, DbResult};

    // The data structures that describe the database schema.
    pub use crate::metadata::{
        AxionDataType,
        // We do not export function-related structs yet as they are not implemented.
        ColumnMetadata,
        DatabaseMetadata,
        EnumMetadata,
        ForeignKeyReference,
        SchemaMetadata,
        TableMetadata,
        ViewMetadata,
    };
}
