// src/api/prism.rs

use axum::Router;
use dev_utils::{debug, info};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::net::TcpListener;

use crate::api::health::{AppState, SharedAppState};

use super::create_health_routes;

/// Configuration options for PrismApi
pub struct PrismConfig<S = String, P = PathBuf>
where
    S: Into<String> + Clone,
    P: Into<PathBuf> + Clone,
{
    pub project_name: S,
    pub version: S,
    pub description: Option<S>,
    pub static_assets_path: Option<P>,
    pub host: S,
    pub port: u16,
}

impl<S, P> PrismConfig<S, P>
where
    S: Into<String> + Clone,
    P: Into<PathBuf> + Clone,
{
    /// Create a new configuration with provided values
    pub fn new(
        project_name: S,
        version: S,
        description: Option<S>,
        static_assets_path: Option<P>,
        host: S,
        port: u16,
    ) -> Self {
        Self {
            project_name,
            version,
            description,
            static_assets_path,
            host,
            port,
        }
    }

    /// Convert generic PrismConfig to concrete String/PathBuf version
    pub fn into_concrete(self) -> PrismConfig<String, PathBuf> {
        PrismConfig {
            project_name: self.project_name.into(),
            version: self.version.into(),
            description: self.description.map(Into::into),
            static_assets_path: self.static_assets_path.map(Into::into),
            host: self.host.into(),
            port: self.port,
        }
    }
}

impl Default for PrismConfig<String, PathBuf> {
    fn default() -> Self {
        Self {
            project_name: "Prism API".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: None,
            static_assets_path: None,
            host: "localhost".into(),
            port: 8080,
        }
    }
}

/// Main PrismApi struct that handles application configuration and setup
pub struct PrismApi {
    // Application configuration - stored as concrete type
    pub config: PrismConfig<String, PathBuf>,

    // Shared application state
    pub state: SharedAppState,
    // Axum app
    // app: Option<Router>,
}

impl Default for PrismApi {
    fn default() -> Self {
        Self {
            config: PrismConfig::default(),
            state: Arc::new(Mutex::new(AppState {
                start_time: SystemTime::now(),
                database_connected: true,
            })),
            // app: None,
        }
    }
}

impl PrismApi {
    /// Create a new PrismApi with custom configuration
    /// Accepts any type that can be converted into the concrete PrismConfig
    pub fn with_config<T, P>(
        // app: Option<Router>,
        config: PrismConfig<T, P>,
    ) -> Self
    where
        T: Into<String> + Clone,
        P: Into<PathBuf> + Clone,
    {
        // Initialize application state
        let state = Arc::new(Mutex::new(AppState {
            start_time: SystemTime::now(),
            database_connected: true, // In a real app, we'd check the database
        }));

        Self {
            config: config.into_concrete(),
            state,
            // app,
        }
    }

    /// Print welcome message with server information
    pub fn print_welcome(&self, host: &str, port: u16) {
        info!("===========================================");
        info!("🚀 {} v{}", self.config.project_name, self.config.version);
        if let Some(desc) = &self.config.description {
            debug!("{}", desc);
        }
        info!("===========================================");
        let address = format!("http://{}:{}", host, port);
        info!("📚 API documentation: {address}/docs");
        info!("📡 Server running at: {address}");
        info!("🏥 Health status: {address}/health");

        info!("===========================================");
    }

    /// Get a reference to the shared application state
    pub fn get_state(&self) -> SharedAppState {
        self.state.clone()
    }

    /// Build the complete application router with proper state handling
    pub fn build_router(&self) -> Router {
        // Create a router without explicit state type first
        let router = Router::new()
            // Nest health routes
            .nest("/health", create_health_routes());

        // Then add the state properly
        router.with_state(self.state.clone())
    }

    // In your prism.rs file, update the serve method
    pub async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Build the router
        let app = self.build_router();

        // Print welcome message before binding
        self.print_welcome(&self.config.host, self.config.port);

        // Use a string format to create a socket address
        let socket_addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| {
                println!(
                    "Warning: Could not parse host '{}', falling back to localhost",
                    self.config.host
                );
                SocketAddr::from(([127, 0, 0, 1], self.config.port))
            });

        println!("Binding to {}", socket_addr);

        // Create the listener with the resolved socket address
        let listener = TcpListener::bind(socket_addr).await?;

        // Serve the application
        axum::serve(listener, app).await?;

        Ok(())
    }
}
