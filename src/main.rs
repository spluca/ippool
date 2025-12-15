mod handlers;
mod ippool;

use axum::{
    Router,
    routing::{delete, get, post},
};
use ippool::IpPool;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Create IP pool with hardcoded values
    let network = "172.16.0".to_string();
    let gateway = "172.16.0.1".to_string();
    let pool = IpPool::new(network, gateway);

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
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    tracing::info!("🚀 IP Pool API server starting with Shuttle");

    // Return the router for Shuttle to serve
    Ok(app.into())
}
