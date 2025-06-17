// axion-db/src/client.rs
use crate::config::DbConfig;
use crate::error::DbResult;
use sqlx::any::AnyPoolOptions;
use sqlx::{AnyPool, Connection};
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct DbClient {
    pub pool: Arc<AnyPool>,
    pub config: Arc<DbConfig>,
}

impl DbClient {
    pub async fn new(config: DbConfig) -> DbResult<Self> {
        info!("Initializing DbClient with config: {:?}", config.db_type);
        let cs = config.build_connection_string()?;
        debug!("Constructed connection string: [REDACTED]");

        let pool_options = if let Some(pool_config) = &config.pool_options {
            AnyPoolOptions::new()
                .max_connections(pool_config.max_connections.unwrap_or(5))
                .min_connections(pool_config.min_connections.unwrap_or(1))
            // Other options...
        } else {
            AnyPoolOptions::new()
        };

        debug!("Connecting to database with type: {:?}", config.db_type);
        let pool = pool_options.connect(&cs).await?;
        info!(
            "Successfully connected to database: {:?}",
            config.database_name.as_deref().unwrap_or("default")
        );

        Ok(Self {
            pool: Arc::new(pool),
            config: Arc::new(config),
        })
    }

    pub async fn test_connection(&self) -> DbResult<()> {
        info!("Pinging database...");
        let mut conn = self.pool.acquire().await?;
        conn.ping().await?;
        info!("Database ping successful.");
        Ok(())
    }

    pub async fn get_db_version(&self) -> DbResult<String> {
        debug!("Fetching database version...");
        let query = match self.config.db_type {
            // Simplified for brevity
            _ => "SELECT version()",
        };
        let (version,): (String,) = sqlx::query_as(query).fetch_one(&*self.pool).await?;
        debug!("Database version: {}", version);
        Ok(version)
    }
}
