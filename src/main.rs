mod handlers;
mod ippool;

use axum::{
    Router,
    routing::{delete, get, post},
};
use clap::Parser;
use ippool::IpPool;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "ippool")]
#[command(about = "IP Pool API server", long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Network prefix (e.g., 172.16.0 for 172.16.0.0/24)
    #[arg(short, long, default_value = "172.16.0")]
    network: String,

    /// Gateway IP address
    #[arg(short, long, default_value = "172.16.0.1")]
    gateway: String,

    /// Enable debug mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("ippool={},tower_http={}", log_level, log_level).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create IP pool
    let pool = IpPool::new(args.network.clone(), args.gateway.clone());
    tracing::info!(
        "🌐 IP Pool initialized: {}.0/24 (Gateway: {})",
        pool.get_network().await,
        pool.get_gateway().await
    );

    // Build application routes
    let app = Router::new()
        // Health check
        .route("/api/v1/health", get(handlers::health_check))
        // IP management - IMPORTANT: Specific routes first, wildcard routes last
        .route("/api/v1/ip/allocate", post(handlers::allocate_ip))
        .route("/api/v1/ip/allocations", get(handlers::list_allocations))
        .route("/api/v1/ip/stats", get(handlers::get_stats))
        .route("/api/v1/ip/release/{vm_id}", delete(handlers::release_ip))
        .route(
            "/api/v1/ip/release-by-ip/{ip}",
            delete(handlers::release_ip_by_address),
        )
        .route("/api/v1/ip/{vm_id}", get(handlers::get_allocation))
        .with_state(pool)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    tracing::info!("🚀 Starting IP Pool API server on {}", addr);
    tracing::info!("📊 API endpoints:");
    tracing::info!("   POST   /api/v1/ip/allocate");
    tracing::info!("   DELETE /api/v1/ip/release/{{vm_id}}");
    tracing::info!("   DELETE /api/v1/ip/release-by-ip/{{ip}}");
    tracing::info!("   GET    /api/v1/ip/{{vm_id}}");
    tracing::info!("   GET    /api/v1/ip/allocations");
    tracing::info!("   GET    /api/v1/ip/stats");
    tracing::info!("   GET    /api/v1/health");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
