# axion-db

Core database interaction layer for the [Axion](https://github.com/Yrrrrrf/axion) framework.

[![GitHub: Axion](https://img.shields.io/badge/GitHub-axion-181717?logo=github)](https://github.com/Yrrrrrf/axion/tree/main/axion-core/axion-db)
[![crates.io](https://img.shields.io/crates/v/axion-db.svg)](https://crates.io/crates/axion-db)
[![docs.rs](https://docs.rs/axion-db/badge.svg)](https://docs.rs/axion-db)
<!-- [![Crates.io Downloads](https://img.shields.io/crates/d/axion-db)](https://crates.io/crates/axion-db) -->

**Note: This crate is part of the Axion framework and is not typically intended for direct standalone use.** It provides the foundational database connectivity, schema introspection, and type mapping capabilities that power Axion.

## Overview

`axion-db` is responsible for:

*   **Database Configuration**: Defining and managing connection parameters for various SQL databases.
*   **Connection Pooling**: Utilizing `sqlx::AnyPool` for efficient, asynchronous database connections.
*   **Schema Introspection**: Querying database metadata (schemas, tables, columns, views, functions, etc.) to understand the database structure.
*   **Type Mapping**: Providing a basic mapping between SQL data types and corresponding Rust types for code generation and dynamic query handling within the Axion framework.
*   **Raw Query Execution**: Offering a simple interface to execute raw SQL queries, primarily used by higher-level Axion components.

## Features

*   **Async Native**: Built on `sqlx` and `tokio` for fully asynchronous database operations.
*   **Multi-Database Support (via `sqlx::Any`):**
    *   PostgreSQL
    *   MySQL/MariaDB
    *   SQLite
*   **Connection Pooling**: Leverages `sqlx`'s robust connection pooling.
*   **Detailed Schema Introspection**: Gathers information about tables, columns (types, nullability, PKs, FKs), views, functions, procedures, and enums.
*   **Configurable**: Flexible `DbConfig` for various connection setups.

## Usage

This crate is primarily consumed by other Axion components (`axion-server` and `axion`). Direct usage would typically involve setting up a `DbConfig` and then creating a `DbClient` to perform introspection or raw queries.

```rust
use axion_db::prelude::*;
use std::sync::Arc;

# async fn run_example() -> axion_db::error::DbResult<()> {
// Ensure drivers are installed for sqlx::Any (typically done once in main application)
sqlx::any::install_default_drivers();

let db_config = DbConfig::new(DatabaseType::Postgres) // Or Mysql, Sqlite
    .host("localhost")
    .port(5432)
    .username("your_user")
    .password("your_password")
    .database_name("your_database")
    .pool_options(PoolOptionsConfig {
        max_connections: Some(10),
        ..Default::default()
    });

let client = Arc::new(DbClient::new(db_config).await?);

client.test_connection().await?;
let version = client.get_db_version().await?;
println!("Connected to database version: {}", version.lines().next().unwrap_or_default());

// Example: List schemas
let schemas = client.list_all_schemas(false).await?; // false = exclude system schemas
println!("Available user schemas: {:?}", schemas);

if let Some(first_schema) = schemas.first() {
    // Example: List tables in the first user schema
    let tables = client.list_tables_in_schema(first_schema).await?;
    println!("Tables in schema '{}': {:?}", first_schema, tables);

    if let Some(first_table) = tables.first() {
        // Example: Get metadata for the first table in that schema
        let table_meta = client.get_table_metadata(first_schema, first_table).await?;
        println!("Metadata for table '{}.{}': {:#?}", first_schema, first_table, table_meta);
    }
}
# Ok(())
# }
```

For more comprehensive examples of how `axion-db` is used to power automatic API generation, please see the main [Axion repository examples](https://github.com/Yrrrrrf/axion/tree/main/axion/examples).
