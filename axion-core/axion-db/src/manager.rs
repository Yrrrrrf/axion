// axion-db/src/manager.rs
use crate::{
    client::DbClient,
    config::DbConfig,
    error::DbResult,
    introspection::{self, Introspector},
    metadata::DatabaseMetadata,
};
use std::sync::Arc;
use tracing::info;

/// The ModelManager is the primary entry point for database introspection.
/// It holds the complete database schema and provides methods to interact with it.
#[derive(Clone)]
pub struct ModelManager {
    pub db_client: Arc<DbClient>,
    pub metadata: Arc<DatabaseMetadata>,
    introspector: Arc<dyn Introspector>,
}

impl ModelManager {
    /// Creates a new ModelManager by connecting to the database and performing a full introspection.
    pub async fn new(config: DbConfig) -> DbResult<Self> {
        info!("Initializing ModelManager...");
        let db_client = Arc::new(DbClient::new(config).await?);
        let introspector = introspection::new_introspector(db_client.clone())?;

        info!("Discovering user schemas...");
        let schemas = introspector.list_user_schemas().await?;

        info!("Performing full database introspection...");
        let metadata = introspector.introspect(&schemas).await?;
        info!(
            "Introspection complete. Found {} schemas.",
            metadata.schemas.len()
        );

        Ok(Self {
            db_client,
            metadata: Arc::new(metadata),
            introspector: Arc::from(introspector),
        })
    }

    // =================================================================================
    //  NEW: Developer Experience (DX) - Pretty-Printing Methods
    // =================================================================================

    /// Prints a high-level summary of all discovered schemas to the console.
    pub fn display_summary(&self) {
        println!("\n{:=<80}", "");
        println!("           DATABASE METADATA SUMMARY");
        println!("{:=<80}\n", "");
        println!("{:#?}", self.metadata); // Uses the custom Debug impl for DatabaseMetadata
    }

    /// Prints a detailed, prism-py-like breakdown of tables for the specified schemas.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_tables(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           TABLES OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display = if schemas.is_empty() {
            self.metadata.schemas.keys().map(|s| s.as_str()).collect()
        } else {
            schemas.to_vec()
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                for table_data in schema_data.tables.values() {
                    // This now uses the beautiful `Display` implementation we wrote for TableMetadata
                    println!("{}\n", table_data);
                }
            }
        }
    }

    /// Prints a detailed, prism-py-like breakdown of views for the specified schemas.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_views(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           VIEWS OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display = if schemas.is_empty() {
            self.metadata.schemas.keys().map(|s| s.as_str()).collect()
        } else {
            schemas.to_vec()
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                for view_data in schema_data.views.values() {
                    // Uses the `Display` implementation for ViewMetadata
                    println!("{}\n", view_data);
                }
            }
        }
    }

    /// Prints a summary of all enums for the specified schemas.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_enums(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           ENUMS OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display = if schemas.is_empty() {
            self.metadata.schemas.keys().map(|s| s.as_str()).collect()
        } else {
            schemas.to_vec()
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                if !schema_data.enums.is_empty() {
                    println!("Schema '{}':", schema_name);
                    for enum_data in schema_data.enums.values() {
                        // Uses the `Display` implementation for EnumMetadata
                        println!("  - {}\n", enum_data);
                    }
                }
            }
        }
    }
}
