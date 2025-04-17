// examples/server_simple.rs
use prismatic::api::PrismApi;
use prismatic::api::prism::PrismConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Create configuration with explicit values
    let config = PrismConfig {
        project_name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        description: Option::Some(env!("CARGO_PKG_DESCRIPTION")),
        static_assets_path: Some(std::path::PathBuf::from("assets")),
        host: "127.0.0.1", // Explicit string conversion
        port: 3000,
    };

    // Create PrismApi with our config
    let prism = PrismApi::with_config(config);

    // Build and serve the application
    prism.serve().await?;

    Ok(())
}
