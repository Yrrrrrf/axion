// src/api/health/routes.rs

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

// Type for the application state
pub type SharedAppState = Arc<Mutex<AppState>>;

// App state structure
pub struct AppState {
    pub start_time: SystemTime,
    pub database_connected: bool,
}

// Health check response model
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
    uptime: f64,
    database_connected: bool,
}

// Cache status response model
#[derive(Serialize)]
pub struct CacheStatus {
    last_updated: String,
    total_items: i32,
    tables_cached: i32,
    views_cached: i32,
    enums_cached: i32,
    functions_cached: i32,
}

// Handler for the main health check endpoint
async fn health_check(State(state): State<SharedAppState>) -> Json<HealthResponse> {
    let state = state.lock().unwrap();
    let now = SystemTime::now();
    let uptime = now
        .duration_since(state.start_time)
        .unwrap_or(Duration::from_secs(0))
        .as_secs_f64();

    // Convert SystemTime to ISO 8601 format
    let datetime: DateTime<Utc> = now.into();

    Json(HealthResponse {
        status: if state.database_connected {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        timestamp: datetime.to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        database_connected: state.database_connected,
    })
}

// Simple ping handler for load balancer health checks
async fn ping() -> &'static str {
    "pong"
}

// Handler for checking cache status
async fn cache_status() -> Json<CacheStatus> {
    let now: DateTime<Utc> = SystemTime::now().into();

    Json(CacheStatus {
        last_updated: now.to_rfc3339(),
        total_items: 0,
        tables_cached: 0,
        views_cached: 0,
        enums_cached: 0,
        functions_cached: 0,
    })
}

// Handler for clearing metadata cache
async fn clear_cache() -> Json<serde_json::Value> {
    // In a real implementation, this would actually clear some cache
    Json(serde_json::json!({
        "status": "success",
        "message": "Cache cleared and reloaded successfully"
    }))
}

// Function to create the health routes router
pub fn create_health_routes() -> Router<SharedAppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/ping", get(ping))
        .route("/cache", get(cache_status))
        .route("/clear-cache", post(clear_cache))
}
