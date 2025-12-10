use crate::ippool::{IpPool, IpPoolError};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

// Error response type
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct AllocateIpRequest {
    pub vm_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AllocateIpResponse {
    pub ip: String,
    pub vm_id: String,
    pub gateway: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseIpResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

// Custom error type for handlers
impl IntoResponse for IpPoolError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            IpPoolError::NoAvailableIps => (
                StatusCode::SERVICE_UNAVAILABLE,
                "No available IPs in pool".to_string(),
            ),
            IpPoolError::IpNotFound => (StatusCode::NOT_FOUND, "IP not found".to_string()),
            IpPoolError::InvalidIp => (StatusCode::BAD_REQUEST, "Invalid IP address".to_string()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

// Health check handler
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

// Allocate IP handler
pub async fn allocate_ip(
    State(pool): State<IpPool>,
    Json(req): Json<AllocateIpRequest>,
) -> Result<(StatusCode, Json<AllocateIpResponse>), IpPoolError> {
    let ip = pool.allocate_ip(req.vm_id.clone()).await?;
    let stats = pool.get_stats().await;

    let response = AllocateIpResponse {
        ip,
        vm_id: req.vm_id,
        gateway: stats["gateway"].as_str().unwrap().to_string(),
        network: stats["network"].as_str().unwrap().to_string(),
        hostname: req.hostname,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// Release IP by VM_ID handler
pub async fn release_ip(
    State(pool): State<IpPool>,
    Path(vm_id): Path<String>,
) -> Result<Json<ReleaseIpResponse>, IpPoolError> {
    pool.release_ip(&vm_id).await?;

    Ok(Json(ReleaseIpResponse {
        message: "IP released successfully".to_string(),
        vm_id: Some(vm_id),
        ip: None,
    }))
}

// Release IP by address handler
pub async fn release_ip_by_address(
    State(pool): State<IpPool>,
    Path(ip): Path<String>,
) -> Result<Json<ReleaseIpResponse>, IpPoolError> {
    pool.release_ip_by_address(&ip).await?;

    Ok(Json(ReleaseIpResponse {
        message: "IP released successfully".to_string(),
        vm_id: None,
        ip: Some(ip),
    }))
}

// Get allocation handler
pub async fn get_allocation(
    State(pool): State<IpPool>,
    Path(vm_id): Path<String>,
) -> Result<Json<crate::ippool::IpAllocation>, IpPoolError> {
    let allocation = pool.get_allocation(&vm_id).await?;
    Ok(Json(allocation))
}

// List allocations handler
pub async fn list_allocations(
    State(pool): State<IpPool>,
) -> Json<Vec<crate::ippool::IpAllocation>> {
    let allocations = pool.list_allocations().await;
    Json(allocations)
}

// Get stats handler
pub async fn get_stats(State(pool): State<IpPool>) -> Json<serde_json::Value> {
    let stats = pool.get_stats().await;
    Json(stats)
}
