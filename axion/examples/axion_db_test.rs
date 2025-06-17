// axion/examples/axion_full_schema_test.rs

use axion_db::{
    error::DbResult,
    introspection::postgres::PostgresIntrospector, // Directly use the Postgres one for this test
    prelude::*,
};
use std::sync::Arc;
use tracing::{error, info, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// A simple macro to pretty-print key-value pairs with indentation.
macro_rules! display_field {
    ($indent:expr, $key:expr, $value:expr) => {
        println!("{}{:<20} : {:?}", $indent, $key, $value);
    };
    ($indent:expr, $key:expr, $value:expr, debug) => {
        println!("{}{:<20} : {:#?}", $indent, $key, $value);
    };
}

// Helper function to list all user-defined schemas
async fn list_all_user_schemas(client: &DbClient) -> DbResult<Vec<String>> {
    let query = "
        SELECT nspname::TEXT AS schema_name
        FROM pg_catalog.pg_namespace
        WHERE nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
          AND nspname NOT LIKE 'pg_temp_%'
        ORDER BY schema_name;
    ";
    let rows: Vec<(String,)> = sqlx::query_as(query)
        .fetch_all(&*client.pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

#[tokio::main]
async fn main() -> DbResult<()> {
    // ---- Boilerplate Setup ----
    sqlx::any::install_default_drivers();
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,axion_db=trace"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting FULL database introspection test...");

    // ---- Configuration ----
    let db_config = DbConfig::new(
        // Forcing Postgres as this test is specific to its introspector
        axion_db::config::DatabaseType::Postgres, 
    )
    .host(std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()))
    .port(std::env::var("DB_PORT").unwrap_or_else(|_| "5432".into()).parse().unwrap())
    .username(std::env::var("DB_OWNER_ADMIN").unwrap_or_else(|_| "a_hub_admin".into()))
    .password(std::env::var("DB_OWNER_PWORD").unwrap_or_else(|_| "password".into()))
    .database_name(std::env::var("DB_NAME").unwrap_or_else(|_| "a_hub".into()));

    // ---- Test Execution ----
    let client = Arc::new(DbClient::new(db_config).await?);
    info!("DbClient created successfully.");
    client.test_connection().await?;

    // 1. Discover all user schemas dynamically
    let all_schemas = list_all_user_schemas(&client).await?;
    info!("Discovered user schemas: {:?}", &all_schemas);

    // 2. Instantiate the Introspector
    let introspector = PostgresIntrospector::new(client.clone());

    // 3. Run introspection on ALL discovered schemas
    let span = span!(Level::INFO, "introspect_full_database");
    let _enter = span.enter();
    
    let full_metadata = match introspector.introspect(&all_schemas).await {
        Ok(meta) => {
            info!("Successfully fetched metadata for {} schemas.", meta.schemas.len());
            meta
        },
        Err(e) => {
            error!("Failed to fetch database metadata: {}", e);
            return Err(e);
        }
    };

    // 4. Display the comprehensive results
    println!("\n{:=<80}", "");
    println!("           COMPLETE DATABASE METADATA OVERVIEW");
    println!("{:=<80}\n", "");

    for schema_name in &all_schemas {
        if let Some(schema_data) = full_metadata.schemas.get(schema_name) {
            println!("Schema: {}", schema_name);
            println!("{:-<80}", "");

            // --- Display Enums ---
            if schema_data.enums.is_empty() {
                println!("  Enums: None");
            } else {
                println!("  Enums ({}):", schema_data.enums.len());
                for (enum_name, enum_data) in &schema_data.enums {
                    println!("    - Enum: {}", enum_name);
                    display_field!("      ", "Values", &enum_data.values);
                }
            }

            // --- Display Views ---
            if schema_data.views.is_empty() {
                println!("\n  Views: None");
            } else {
                println!("\n  Views ({}):", schema_data.views.len());
                for (view_name, view_data) in &schema_data.views {
                    println!("    - View: {}", view_name);
                    // Optionally display view definition or column count
                    display_field!("      ", "Columns", view_data.columns.len());
                }
            }
            
            // --- Display Tables ---
            if schema_data.tables.is_empty() {
                println!("\n  Tables: None");
            } else {
                println!("\n  Tables ({}):", schema_data.tables.len());
                for (table_name, table_data) in &schema_data.tables {
                    println!("    - Table: {}", table_name);
                    display_field!("      ", "Primary Keys", &table_data.primary_key_columns);
                    println!("      Columns:");
                    for col in &table_data.columns {
                        println!("        > Column: {}", col.name);
                        display_field!("          ", "Axion Type", &col.axion_type, debug);
                        display_field!("          ", "Nullable", &col.is_nullable);
                        if col.foreign_key.is_some() {
                           display_field!("          ", "Foreign Key", &col.foreign_key);
                        }
                    }
                }
            }
            println!("\n");
        }
    }
    
    info!("Full introspection test completed successfully.");
    Ok(())
}