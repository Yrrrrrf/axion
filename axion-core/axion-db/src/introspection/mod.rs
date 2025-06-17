// axion-db/src/introspection/mod.rs
use crate::{
    client::DbClient,
    config::DatabaseType,
    error::{DbError, DbResult},
    metadata::{DatabaseMetadata, EnumMetadata, SchemaMetadata, TableMetadata, ViewMetadata},
};
use std::{collections::HashMap, sync::Arc};

// --- Implementations for each dialect ---
pub mod postgres;
// pub mod mysql; // Future

/// The main Introspector trait that all database-specific introspectors must implement.
#[async_trait::async_trait]
pub trait Introspector: Send + Sync {
    async fn list_user_schemas(&self) -> DbResult<Vec<String>>;
    async fn introspect(&self, schemas: &[String]) -> DbResult<DatabaseMetadata>;
    async fn introspect_schema(&self, schema_name: &str) -> DbResult<SchemaMetadata>;
    async fn introspect_table(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> DbResult<TableMetadata>;
    async fn introspect_view(&self, schema_name: &str, view_name: &str) -> DbResult<ViewMetadata>;
    async fn introspect_enums_for_schema(
        &self,
        schema_name: &str,
    ) -> DbResult<HashMap<String, EnumMetadata>>;
}

// ==============================================================================
//  The Dispatcher Macro and Enum
// ==============================================================================

/// A factory function that creates the correct, boxed introspector based on the database dialect.
pub fn new_introspector(client: Arc<DbClient>) -> DbResult<Box<dyn Introspector>> {
    match client.config.db_type {
        DatabaseType::Postgres => Ok(Box::new(postgres::PostgresIntrospector::new(client))),
        // Future dialects would be added here:
        // DatabaseType::Mysql => Ok(Box::new(mysql::MySqlIntrospector::new(client))),
        _ => Err(DbError::UnsupportedDbType(
            "This database type is not yet supported for introspection.".to_string(),
        )),
    }
}
