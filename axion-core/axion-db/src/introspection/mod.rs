// in axion-db/src/introspection/mod.rs

use crate::{
    error::DbResult,
    // Add ViewMetadata and EnumMetadata to the imports
    metadata::{DatabaseMetadata, EnumMetadata, SchemaMetadata, TableMetadata, ViewMetadata},
};
use std::collections::HashMap; // Add HashMap import

pub mod postgres;

/// A trait for introspecting a database schema.
#[async_trait::async_trait]
pub trait Introspector: Send + Sync {
    /// Introspect the entire database for the given schemas.
    async fn introspect(&self, schemas: &[String]) -> DbResult<DatabaseMetadata>;

    /// Introspect a single schema, including its tables, views, and enums.
    async fn introspect_schema(&self, schema_name: &str) -> DbResult<SchemaMetadata>;

    /// Introspect a single table within a schema.
    async fn introspect_table(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> DbResult<TableMetadata>;

    /// Introspect a single view within a schema.
    async fn introspect_view(&self, schema_name: &str, view_name: &str) -> DbResult<ViewMetadata>;

    /// Introspect all user-defined enums within a schema.
    async fn introspect_enums_for_schema(
        &self,
        schema_name: &str,
    ) -> DbResult<HashMap<String, EnumMetadata>>;
}
