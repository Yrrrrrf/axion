<h1 align="center">
<img src="https://raw.githubusercontent.com/Yrrrrrf/axion/main/resources/img/arrow.png" alt="Axion Icon" width="128" height="128" description="An icon representing Axion: transforming a single data source (database) into a spectrum of API endpoints.">
<div align="center">AXION</div>
</h1>

<div align="center">

[![crates.io](https://img.shields.io/crates/v/axion.svg)](https://crates.io/crates/axion)
[![GitHub: Axion](https://img.shields.io/badge/GitHub-axion-181717?logo=github)](https://github.com/Yrrrrrf/axion)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://choosealicense.com/licenses/mit/)

</div>

High-performance Rust library for automatic API generation from database schemas.It empowers you to create a blazingly fast REST API that mirrors your database structure, effortlessly handling tables, views, functions, and procedures with Rust's renowned memory safety and zero-cost abstractions. Focused on speed and resource efficiency, it's the ideal solution for microservices and high-traffic APIs.

The name axion not only hints at its foundation on [Axum](https://axum.rs/) but also resonates with the Spanish word "*acción*" (action), reflecting its core promise: to take decisive action in transforming your **database schema into a fully functional API** with minimal effort.


> Note: This library is part of a broader ecosystem aimed at simplifying database-to-client workflows, which also includes Python ([prism-py](https://github.com/Yrrrrrf/prism-py)) and TypeScript ([prism-ts](https://github.com/Yrrrrrf/prism-ts)) variants. **Axion represents the high-performance Rust engine** within this ecosystem.

> Note: This is an early version of the library. While functional, it may not be fully stable. Please report any issues you encounter.

# Key Features

🚀 High Performance: Built on [Tokio](https://tokio.rs/) and [Hyper](https://hyper.rs/) for maximum throughput.  
🔒 Memory Safety: Rust's ownership model guarantees thread safety.  
⚡ Async Everything: Fully asynchronous I/O operations without blocking.  
🔄 Automatic Route Generation: Create CRUD endpoints for database objects.  
🌐 Database Independence: Support for PostgreSQL, MySQL, and SQLite.  
🧩 Schema-Based Organization: Routes organized by database schemas for clean API structure.  
📊 Enhanced Filtering: Sorting, pagination, and complex query support.  
🔍 Metadata API: Explore your database structure programmatically.  
🏥 Health Monitoring: Built-in health check endpoints.  
📦 Zero Boilerplate: **Generate complete APIs with minimal code – true "acción"!**  
📉 Low Resource Usage: Minimal memory footprint and CPU utilization.  

# Installation
Add to your Cargo.toml
```sh
cargo add axion
```

# Quick Start

Here's a minimal example to get you started with axion:

```rust
use axion::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logs
    tracing_subscriber::fmt::init();

    // Configure database connection
    let db_config = DbConfig::new()
        .database_type(DatabaseType::Postgres) // Example
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
    let axion_api = AxionApi::new(model_manager.clone());
    
    // Build router with all generated routes
    let app = axion_api.build_router();
    
    // Serve the application
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

# Plans for the Future
- Full feature parity with prism-py and prism-ts.
- Comprehensive support for all database types (PostgreSQL, MySQL, SQLite, etc.).
- Support for all query types (CRUD, JOIN, etc.).
- Robust authentication mechanisms (JWT, OAuth, etc.).
- Flexible authorization models (RBAC, ABAC, etc.).
- Advanced error handling strategies (HTTP, custom, etc.).
- Versatile logging options (file, console, etc.).
- Customizable API routes and endpoints.
- Advanced filtering and sorting capabilities.
- Custom pagination solutions.
- Support for various response formats (e.g., JSON, XML, etc.).
- Custom request validation and sanitization.

<!--
## Generated Routes (Example - To be defined for Axion)

Axion will automatically create the following types of routes:

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

See the `examples` directory for complete sample applications (once available):
-->

# Performance

axion is engineered for exceptional performance and minimal resource usage:

- Aims for significantly faster response times compared to equivalent dynamic language implementations.
- Minimal memory footprint with efficient connection pooling.
- High concurrency, capable of handling thousands of simultaneous connections.
- Low CPU usage, even under heavy load.

# License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
