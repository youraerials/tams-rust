mod auth;
mod config;
mod database;
mod error;
mod handlers;
mod models;
mod storage;
mod time_utils;
mod webhooks;

use crate::{
    auth::{auth_middleware, AuthState},
    config::AppConfig,
    database::Database,
    handlers::{*, AppState, AppStateInner},
    storage::MediaStorage,
    webhooks::WebhookManager,
};
use axum::{
    http::Method,
    middleware,
    routing::{delete, get, head, post, put},
    Router,
};

use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// AppState is defined in handlers.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config = AppConfig::new().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        e
    })?;

    // Initialize logging
    init_logging(&config.logging.level, &config.logging.format)?;
    info!("Starting TAMS Rust server...");

    // Initialize database
    info!("Initializing database...");
    let database = Arc::new(Database::new(&config.database.url, config.database.max_connections).await?);
    database.migrate().await?;
    info!("Database initialized successfully");

    // Initialize media storage
    info!("Initializing media storage...");
    let storage = Arc::new(MediaStorage::new(
        config.media_storage.clone(),
        config.service.public_url_base.clone(),
    )?);
    storage.ensure_directories().await?;
    info!("Media storage initialized successfully");

    // Initialize webhook manager
    info!("Initializing webhook manager...");
    let webhook_manager = Arc::new(WebhookManager::new());
    
    // Load existing webhooks from database
    let _webhooks = database.get_webhooks_list().await?;
    // Note: WebhookManager::new() doesn't need pre-loaded webhooks
    info!("Webhook manager initialized");

    // Create application state
    let app_state = Arc::new(AppStateInner {
        config,
        database: (*database).clone(),
        storage,
        webhook_manager,
    });

    // Create auth state  
    let auth_state = Arc::new(AuthState::new(app_state.config.auth.clone()));

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
        ])
        .allow_origin(Any)
        .allow_headers(Any);

    // Build the application routes
    let app = Router::new()
        // Root endpoints
        .route("/", get(get_root))
        .route("/service", get(get_service_info))
        .route("/test", get(get_test_page))
        
        // Sources endpoints
        .route("/sources", get(list_sources).post(create_source))
        .route("/sources/:source_id", 
            get(get_source)
                .put(update_source)
                .delete(delete_source)
        )
        
        // Flows endpoints
        .route("/flows", get(list_flows).post(create_flow))
        .route("/flows/:flow_id", 
            get(get_flow)
                .put(update_flow)
                .delete(delete_flow)
        )
        
        // Flow segments endpoints
        .route("/flows/:flow_id/segments", 
            get(list_flow_segments)
                .post(add_flow_segment)
                .delete(delete_flow_segments)
        )
        
        // Flow storage endpoints
        .route("/flows/:flow_id/storage", get(allocate_storage))
        
        // Media objects endpoints
        .route("/objects/:object_id", 
            get(get_media_object)
        )
        
        // Webhook endpoints
        .route("/service/webhooks", 
            get(list_webhooks)
                .post(create_webhook)
        )
        
        // Flow delete request endpoints
        .route("/flow-delete-requests", 
            get(list_deletion_requests)
                .post(request_flow_deletion)
        )
        .route("/flow-delete-requests/:request_id", get(get_deletion_request))
        
        // Add application state
        .with_state(app_state.clone())
        
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(middleware::from_fn_with_state(
                    auth_state.clone(),
                    auth_middleware,
                ))
        );

    // Create server address
    let addr = SocketAddr::from((
        app_state.config.server.host.parse::<std::net::IpAddr>()?,
        app_state.config.server.port,
    ));

    info!("TAMS server starting on {}", addr);
    info!("Service: {} v{}", app_state.config.service.name, app_state.config.service.version);
    info!("Authentication: {}", if app_state.config.auth.require_auth { "enabled" } else { "disabled" });
    info!("Media storage: {}", app_state.config.media_storage.base_path.display());
    info!("Database: {}", app_state.config.database.url);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("TAMS server starting on {}", addr);
    info!("API Documentation: {}/", addr);
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("TAMS server stopped");
    Ok(())
}

fn init_logging(level: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    match format {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
        "pretty" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down...");
        },
    }
} 