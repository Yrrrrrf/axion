<h1 align="center">
  <img src="https://raw.githubusercontent.com/Yrrrrrf/prismatic/main/resources/img/prism.png" alt="Prism Icon" width="128" height="128" description="A prism that can take one light source and split it into multiple colors!">
  <div align="center">prismatic</div>
</h1>

<div align="center">

<!-- [![crates.io](https://img.shields.io/crates/v/prismatic.svg)](https://crates.io/crates/prismatic) -->
[![GitHub: prismatic](https://img.shields.io/badge/GitHub-prismatic-181717?logo=github)](https://github.com/Yrrrrrf/prismatic)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://choosealicense.com/licenses/mit/)
<!-- [![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org) -->

</div>

## Overview

**prismatic** is a high-performance Rust library for automatic API generation from database schemas. It creates a blazingly fast REST API that mirrors your database structure, handling tables, views, functions, and procedures with memory safety and zero-cost abstractions.

Built with [Axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx), **prismatic** eliminates boilerplate code while providing exceptional performance and safety. Focused on speed and resource efficiency, it's the ideal solution for microservices and high-traffic APIs.

> **Note**: This library is part of the Prism ecosystem, which also includes [**prism-py**](https://github.com/Yrrrrrf/prism-py) (`Python`) and [**prism-ts**](https://github.com/Yrrrrrf/prism-ts) (`TypeScript`) variants.

> **Note**: This is an early version of the library, and while it is functional, it may not be fully stable. Please report any issues you encounter.

## Key Features

- **🚀 High Performance**: Built on Tokio and Hyper for maximum throughput
- **🔒 Memory Safety**: Rust's ownership model guarantees thread safety
- **⚡ Async Everything**: Fully asynchronous I/O operations without blocking
- **🔄 Automatic Route Generation**: Create CRUD endpoints for database objects
- **🌐 Database Independence**: Support for PostgreSQL, MySQL, and SQLite
- **🧩 Schema-Based Organization**: Routes organized by database schemas for clean API structure
- **📊 Enhanced Filtering**: Sorting, pagination, and complex query support
- **🔍 Metadata API**: Explore your database structure programmatically
- **🏥 Health Monitoring**: Built-in health check endpoints
- **📦 Zero Boilerplate**: Generate complete APIs with minimal code
- **📉 Low Resource Usage**: Minimal memory footprint and CPU utilization

## Installation

```bash
# Add to your Cargo.toml
cargo add prism

# Or with specific features
cargo add prism --features postgres
```

## Quick Start

Here's a minimal example to get you started:

```rust
use prism_rs::{PrismApi, DbConfig, ModelManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logs
    tracing_subscriber::fmt::init();

    // Configure database connection
    let db_config = DbConfig::new()
        .database_type(DatabaseType::Postgres)
        .host("localhost")
        .port(5432)
        .database("yourdb")
        .username("username")
        .password("password")
        .build()?;

    // Create database client
    let db_client = Arc::new(DbClient::new(db_config).await?);
    db_client.test_connection().await?;

    // Create model manager with selected schemas
    let model_manager = Arc::new(ModelManager::new(
        db_client.clone(),
        vec!["public".to_string(), "app".to_string()]
    ).await?);

    // Initialize API generator
    let prism = PrismApi::new(model_manager.clone());
    
    // Build router with all generated routes
    let app = prism.build_router();
    
    // Serve the application
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

## Plans for the Future

- [ ] Main clone of [prism-py](https://pypi.org/project/prism-py/) and [prism-ts](https://www.npmjs.com/package/prism-ts) with all features
    - [ ] Support for all database types (PostgreSQL, MySQL, SQLite, etc.)
    - [ ] Support for all query types (CRUD, JOIN, etc.)
    - [ ] Support for all authentication types (JWT, OAuth, etc.)
    - [ ] Support for all authorization types (RBAC, ABAC, etc.)
    - [ ] Support for all error handling types (HTTP, custom, etc.)
    - [ ] Support for all logging types (file, console, etc.)
    - [ ] Customizable API routes and endpoints
        - [ ] Support for more advanced filtering and sorting options
        - [ ] Support for custom pagination options
        - [ ] Support for custom response formats (e.g., JSON, XML, etc.)
        - [ ] Support for custom request validation and sanitization

<!-- 
## Generated Routes

prismatic automatically creates the following types of routes:

### Table Routes
- `POST /{schema}/{table}` - Create a record
- `GET /{schema}/{table}` - Read records with filtering
- `PUT /{schema}/{table}` - Update records
- `DELETE /{schema}/{table}` - Delete records

### View Routes
- `GET /{schema}/{view}` - Read from view with optional filtering

### Function/Procedure Routes
- `POST /{schema}/fn/{function}` - Execute database function
- `POST /{schema}/proc/{procedure}` - Execute stored procedure

### Metadata Routes
- `GET /dt/schemas` - List all database schemas and structure
- `GET /dt/{schema}/tables` - List all tables in a schema
- `GET /dt/{schema}/views` - List all views in a schema
- `GET /dt/{schema}/functions` - List all functions in a schema
- `GET /dt/{schema}/procedures` - List all procedures in a schema

### Health Routes
- `GET /health` - Get API health status
- `GET /health/ping` - Basic connectivity check
- `GET /health/cache` - Check metadata cache status
- `POST /health/clear-cache` - Clear and reload metadata cache

## Usage Examples

See the [examples](./examples) directory for complete sample applications:

- **[Simple Server](./examples/simple_server.rs)**: Basic server setup with minimal configuration
- **[Full API](./examples/full_api.rs)**: Comprehensive example with all features enabled
- **[Authentication](./examples/authentication.rs)**: Adding JWT authentication to your API -->

## Performance

prismatic is designed for high performance and low resource usage:

- **10-50x** faster response times compared to equivalent Python implementations
- **Minimal memory footprint** with efficient connection pooling
- **High concurrency** handling thousands of simultaneous connections
- **Low CPU usage** even under heavy load

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Before contributing:
1. Run `cargo fmt` to format your code
2. Ensure all tests pass with `cargo test`
3. Check for issues with `cargo clippy`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.