#![allow(unused)]
// axion/examples/axion_db_test.rs


use axion_db::prelude::*;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> DbResult<()> {
    // ---- Boilerplate Setup ----
    sqlx::any::install_default_drivers();
    dotenvy::dotenv().ok();

    // Using the .pretty() layer for beautiful logs!
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,axion_db=trace"))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // ---- Configuration ----
    let db_config = DbConfig::new(DatabaseType::Postgres)
        .host(std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()))
        .port(
            std::env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".into())
                .parse()
                .unwrap(),
        )
        .username(std::env::var("DB_OWNER_ADMIN").unwrap_or_else(|_| "a_hub_admin".into()))
        .password(std::env::var("DB_OWNER_PWORD").unwrap_or_else(|_| "password".into()))
        .database_name(std::env::var("DB_NAME").unwrap_or_else(|_| "a_hub".into()));

    // ---- Test Execution ----

    // 1. Create the ModelManager. This one line handles connection and full introspection.
    let model_manager = match ModelManager::new(db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize ModelManager: {}", e);
            return Err(e);
        }
    };

    // 2. Use the new, high-level display methods.
    //    This is the clean, `prism-py`-like API we wanted.
    let schemas = [
        //* */ Default schemas
        // 'public',  # * This is the default schema
        "account",
        "auth",
        // * A-Hub schemas
        "agnostic",
        "infrastruct",
        "hr",
        "academic",
        "course_offer",
        "student",
        "library",
    ];

    // Display a summary of everything found.
    // model_manager.display_summary();

    // Display a detailed breakdown of tables from specific schemas.
    // model_manager.display_tables(&schemas);

    // // Display a detailed breakdown of all discovered views.
    model_manager.display_views(&[]); // Empty slice means "all schemas"

    // Display a summary of all discovered enums.
    model_manager.display_enums(&[]);

    Ok(())
}
