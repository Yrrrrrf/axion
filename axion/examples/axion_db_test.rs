// axion/examples/axion_db_test.rs
use axion_db::prelude::*;
use std::str::FromStr; // Required for DatabaseType::from_str
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> DbResult<()> {
    // --- THIS IS THE FIX ---
    // Install default SQLx drivers (Postgres, MySQL, SQLite if features are enabled)
    // This needs to be called once, typically at the start of your application.
    sqlx::any::install_default_drivers();
    // --- END FIX ---

    // Load environment variables from .env file if present
    // dotenvy::dotenv().ok();

    // Setup tracing (optional, but good for debugging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,axion_db=trace", // Adjust log levels
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Construct DbConfig from individual environment variables ---
    let db_type_str = std::env::var("DB_TYPE").unwrap_or_else(|_| "postgresql".to_string());
    let db_type = DatabaseType::from_str(&db_type_str)
        .map_err(|e| DbError::Config(format!("Invalid DB_TYPE: {}", e)))?;

    let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());

    let port_str = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let port = port_str
        .parse::<u16>()
        .map_err(|e| DbError::Config(format!("Invalid DB_PORT '{}': {}", port_str, e)))?;

    let username = std::env::var("DB_OWNER_ADMIN").unwrap_or_else(|_| "a_hub_admin".to_string());
    let password = std::env::var("DB_OWNER_PWORD").unwrap_or_else(|_| "password".to_string());
    let database_name = std::env::var("DB_NAME").unwrap_or_else(|_| "a_hub".to_string());
    // --- End DbConfig construction ---

    let db_config = DbConfig::new(db_type)
        .host(host)
        .port(port)
        .username(username)
        .password(password)
        .database_name(database_name)
        .pool_options(PoolOptionsConfig {
            max_connections: Some(5),
            min_connections: Some(1),
            connect_timeout_seconds: Some(30),
            idle_timeout_seconds: Some(300),
            max_lifetime_seconds: Some(1800),
            acquire_timeout_seconds: Some(30),
            test_before_acquire: Some(true),
        });

    println!(
        "Attempting to connect to database using config: {:?}",
        db_config.db_type
    );
    let client = Arc::new(DbClient::new(db_config).await?);
    println!("Successfully created DbClient.");

    println!("\nTesting connection...");
    client.test_connection().await?;
    println!("Connection test successful!");

    let version = client.get_db_version().await?;
    println!(
        "\nDatabase version: {}",
        version.lines().next().unwrap_or(&version)
    );

    println!("\nListing all non-system schemas:");
    let schemas = client.list_all_schemas(false).await?;
    if schemas.is_empty() {
        println!("No user-defined schemas found. Example might be less interesting.");
    }
    for schema in &schemas {
        println!("- {}", schema);
    }

    let schema_to_inspect = std::env::var("DB_SCHEMA_TO_INSPECT").unwrap_or_else(|_| {
        schemas
            .first()
            .map(String::as_str)
            .unwrap_or("public")
            .to_string()
    });
    println!("\nTables in schema '{}':", schema_to_inspect);
    match client.list_tables_in_schema(&schema_to_inspect).await {
        Ok(tables) => {
            if tables.is_empty() {
                println!("No tables found in schema '{}'.", schema_to_inspect);
            }
            for table_name in tables {
                println!("  - {}", table_name);
                match client
                    .get_table_metadata(&schema_to_inspect, &table_name)
                    .await
                {
                    Ok(meta) => {
                        println!("    Table Comment: {:?}", meta.comment);
                        println!("    Primary Keys: {:?}", meta.primary_key_columns);
                        for col in meta.columns {
                            println!(
                                "      - Col: {}, SQL Type: {} (UDT: {:?}), Rust Type: {}, Nullable: {}, PK: {}, Default: {:?}, MaxLen: {:?}, NumP: {:?}, NumS: {:?}, FK: {:?}",
                                col.name,
                                col.sql_type_name,
                                col.udt_name,
                                col.rust_type_string,
                                col.is_nullable,
                                col.is_primary_key,
                                col.default_value,
                                col.character_maximum_length,
                                col.numeric_precision,
                                col.numeric_scale,
                                col.foreign_key_reference,
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "    Error getting metadata for {}.{}: {}",
                            schema_to_inspect, table_name, e
                        )
                    }
                }
            }
        }
        Err(e) => eprintln!(
            "Error listing tables for schema {}: {}",
            schema_to_inspect, e
        ),
    }

    println!("\nFetching full database metadata for non-system schemas...");
    let schemas_to_include_for_full_meta = None;

    match client
        .get_full_database_metadata(schemas_to_include_for_full_meta)
        .await
    {
        Ok(full_metadata) => {
            println!(
                "Fetched metadata for {} schemas.",
                full_metadata.schemas.len()
            );
            for (schema_name, schema_data) in &full_metadata.schemas {
                println!("Schema: {}", schema_name);
                println!("  Tables ({}):", schema_data.tables.len());
                for (table_name, _table_data) in &schema_data.tables {
                    // _table_data to silence warning
                    println!("    - {}", table_name);
                }
            }
        }
        Err(e) => eprintln!("Error fetching full database metadata: {}", e),
    }

    Ok(())
}
