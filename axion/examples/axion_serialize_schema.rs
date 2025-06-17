// axion/examples/axion_serialize_schema.rs
use axion_db::introspection::Introspector;
use axion_db::{
    client::DbClient, error::DbResult, introspection::postgres::PostgresIntrospector, prelude::*,
};
use std::{fs, io::Write, path::Path, sync::Arc};
use tracing::{Level, info, span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Helper function to list all user-defined schemas (same as before)
async fn list_all_user_schemas(client: &DbClient) -> DbResult<Vec<String>> {
    let query = "
        SELECT nspname::TEXT AS schema_name
        FROM pg_catalog.pg_namespace
        WHERE nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
          AND nspname NOT LIKE 'pg_temp_%'
        ORDER BY schema_name;
    ";
    let rows: Vec<(String,)> = sqlx::query_as(query).fetch_all(&*client.pool).await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Using anyhow::Result for easier error handling with file I/O
    // ---- Boilerplate Setup ----
    sqlx::any::install_default_drivers();
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,axion_db=trace"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let env_vars = [
    //     "DB_HOST",
    //     "DB_PORT",
    //     "DB_OWNER_ADMIN",
    //     "DB_OWNER_PWORD",
    //     "DB_NAME",
    // ];
    // for var in &env_vars {
    //     if std::env::var(var).is_err() {
    //         error!("Environment variable {} is not set. Please ensure it is defined.", var);
    //         return Err(anyhow::anyhow!("Missing environment variable: {}", var));
    //     }
    // }

    info!("Starting database introspection and serialization...");

    // ---- Configuration ----
    let db_config = DbConfig::new(axion_db::config::DatabaseType::Postgres)
        .host(std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()))
        .port(
            std::env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".into())
                .parse()?,
        )
        .username(std::env::var("DB_OWNER_ADMIN").unwrap_or_else(|_| "a_hub_admin".into()))
        .password(std::env::var("DB_OWNER_PWORD").unwrap_or_else(|_| "password".into()))
        .database_name(std::env::var("DB_NAME").unwrap_or_else(|_| "a_hub".into()));

    // ---- Introspection ----
    let client = Arc::new(DbClient::new(db_config).await?);
    let all_schemas = list_all_user_schemas(&client).await?;
    let introspector = PostgresIntrospector::new(client.clone());

    let span = span!(Level::INFO, "introspect_and_serialize");
    let _enter = span.enter();

    info!("Fetching metadata for all user schemas...");
    let full_metadata = introspector.introspect(&all_schemas).await?;
    info!("Introspection complete.");

    // ========================================================================
    //  NEW: Serialization Logic
    // ========================================================================

    // 1. Define the output path
    let output_dir = Path::new("temp");
    let output_path = output_dir.join("db_schema.json");

    // 2. Ensure the output directory exists
    if !output_dir.exists() {
        info!("Creating output directory: {:?}", output_dir);
        fs::create_dir_all(output_dir)?;
    }

    // 3. Serialize the metadata to a pretty-printed JSON string
    info!("Serializing metadata to JSON...");
    let json_output = serde_json::to_string_pretty(&full_metadata)?;
    info!(
        "Serialization complete. JSON size: {} bytes",
        json_output.len()
    );

    // 4. Write the JSON string to the file
    info!("Writing schema to file: {:?}", &output_path);
    let mut file = fs::File::create(&output_path)?;
    file.write_all(json_output.as_bytes())?;

    info!(
        "Successfully serialized database schema to {:?}",
        output_path.canonicalize()?
    );

    Ok(())
}
