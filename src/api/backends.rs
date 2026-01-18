//! Backend management API endpoints.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::backend::registry::BackendInfo;

use super::auth::AuthUser;
use super::routes::AppState;

/// Backend information returned by API
#[derive(Debug, Clone, Serialize)]
pub struct BackendResponse {
    pub id: String,
    pub name: String,
}

impl From<BackendInfo> for BackendResponse {
    fn from(info: BackendInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
        }
    }
}

/// Agent information returned by API
#[derive(Debug, Clone, Serialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
}

/// List all available backends
pub async fn list_backends(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthUser>,
) -> Json<Vec<BackendResponse>> {
    let registry = state.backend_registry.read().await;
    let backends: Vec<BackendResponse> = registry.list().into_iter().map(Into::into).collect();
    Json(backends)
}

/// Get a specific backend by ID
pub async fn get_backend(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<BackendResponse>, (StatusCode, String)> {
    let registry = state.backend_registry.read().await;
    match registry.get(&id) {
        Some(backend) => Ok(Json(BackendResponse {
            id: backend.id().to_string(),
            name: backend.name().to_string(),
        })),
        None => Err((StatusCode::NOT_FOUND, format!("Backend {} not found", id))),
    }
}

/// List agents for a specific backend
pub async fn list_backend_agents(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AgentResponse>>, (StatusCode, String)> {
    let registry = state.backend_registry.read().await;
    let backend = registry
        .get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Backend {} not found", id)))?;

    match backend.list_agents().await {
        Ok(agents) => {
            let agents: Vec<AgentResponse> = agents
                .into_iter()
                .map(|a| AgentResponse {
                    id: a.id,
                    name: a.name,
                })
                .collect();
            Ok(Json(agents))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list agents: {}", e),
        )),
    }
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
}

/// Get backend configuration
pub async fn get_backend_config(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<BackendConfig>, (StatusCode, String)> {
    let registry = state.backend_registry.read().await;
    let backend = registry
        .get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Backend {} not found", id)))?;

    // Return backend-specific configuration
    let settings = match id.as_str() {
        "opencode" => {
            let base_url = std::env::var("OPENCODE_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4096".to_string());
            let default_agent = std::env::var("OPENCODE_DEFAULT_AGENT").ok();
            let permissive = std::env::var("OPENCODE_PERMISSIVE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            serde_json::json!({
                "base_url": base_url,
                "default_agent": default_agent,
                "permissive": permissive,
            })
        }
        "claudecode" => {
            // Check if Claude Code API key is configured
            let api_key_configured = state
                .secrets
                .as_ref()
                .map(|s| {
                    // Check async context
                    false // TODO: implement proper secret check
                })
                .unwrap_or(false);
            serde_json::json!({
                "api_key_configured": api_key_configured,
            })
        }
        _ => serde_json::json!({}),
    };

    Ok(Json(BackendConfig {
        id: backend.id().to_string(),
        name: backend.name().to_string(),
        enabled: true,
        settings,
    }))
}

/// Request to update backend configuration
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBackendConfigRequest {
    pub settings: serde_json::Value,
}

/// Update backend configuration
pub async fn update_backend_config(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(_req): Json<UpdateBackendConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let registry = state.backend_registry.read().await;
    if registry.get(&id).is_none() {
        return Err((StatusCode::NOT_FOUND, format!("Backend {} not found", id)));
    }

    // Backend configuration is currently read from environment variables
    // TODO: Implement persistent backend configuration storage

    Ok(Json(serde_json::json!({
        "ok": true,
        "message": "Backend configuration is currently read-only"
    })))
}
