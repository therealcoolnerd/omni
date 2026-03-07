use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use crate::brain::OmniBrain;
use crate::database::Database;
use crate::search::SearchEngine;

#[derive(Clone)]
pub struct AppState {
    brain: Arc<Mutex<OmniBrain>>,
    db: Arc<Database>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    os: String,
    arch: String,
    hostname: String,
    available_managers: Vec<String>,
}

#[derive(Serialize)]
pub struct PackageInfo {
    name: String,
    version: String,
    box_type: String,
    description: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    status: String,
    data: Option<T>,
    message: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { status: "success".into(), data: Some(data), message: None }
    }
    fn err(msg: impl Into<String>) -> ApiResponse<()> {
        ApiResponse { status: "error".into(), data: None, message: Some(msg.into()) }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize { 20 }

#[derive(Deserialize)]
pub struct InstallPayload {
    package: String,
    box_type: Option<String>,
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let brain = OmniBrain::new();
    let db = Database::new().await?;

    let state = AppState {
        brain: Arc::new(Mutex::new(brain)),
        db: Arc::new(db),
    };

    let app = Router::new()
        .route("/api/system/info", get(get_system_info))
        .route("/api/packages/installed", get(get_installed_packages))
        .route("/api/packages/search", get(search_packages))
        .route("/api/packages/install", post(install_package))
        .route("/api/packages/remove", post(remove_package))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    println!("Omni server running on http://{}", addr);
    println!("Web dashboard: http://{}  (open in browser)", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_system_info() -> Json<ApiResponse<SystemInfo>> {
    let available_managers = crate::distro::get_available_package_managers()
        .iter()
        .map(|s| s.to_string())
        .collect();

    Json(ApiResponse::ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into()),
        available_managers,
    }))
}

async fn get_installed_packages(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<PackageInfo>>> {
    match state.db.get_installed_packages().await {
        Ok(records) => {
            let packages = records
                .into_iter()
                .map(|r| PackageInfo {
                    name: r.package_name,
                    version: r.version.unwrap_or_else(|| "unknown".into()),
                    box_type: r.box_type,
                    description: None,
                })
                .collect();
            Json(ApiResponse::ok(packages))
        }
        Err(e) => Json(ApiResponse {
            status: "error".into(),
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

async fn search_packages(
    State(_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<PackageInfo>>> {
    match SearchEngine::new().await {
        Ok(engine) => match engine.search_all(&query.q).await {
            Ok(results) => {
                let packages = results
                    .into_iter()
                    .take(query.limit)
                    .map(|r| PackageInfo {
                        name: r.name,
                        version: r.version.unwrap_or_else(|| "latest".into()),
                        box_type: r.box_type,
                        description: r.description,
                    })
                    .collect();
                Json(ApiResponse::ok(packages))
            }
            Err(e) => Json(ApiResponse {
                status: "error".into(),
                data: None,
                message: Some(e.to_string()),
            }),
        },
        Err(e) => Json(ApiResponse {
            status: "error".into(),
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

async fn install_package(
    State(state): State<AppState>,
    Json(payload): Json<InstallPayload>,
) -> Json<serde_json::Value> {
    let mut brain = state.brain.lock().await;
    match brain.install(&payload.package, payload.box_type.as_deref()).await {
        Ok(_) => serde_json::json!({ "status": "success", "message": format!("Installed {}", payload.package) }).into(),
        Err(e) => serde_json::json!({ "status": "error", "message": e.to_string() }).into(),
    }
}

async fn remove_package(
    State(state): State<AppState>,
    Json(payload): Json<InstallPayload>,
) -> Json<serde_json::Value> {
    let mut brain = state.brain.lock().await;
    match brain.remove(&payload.package, payload.box_type.as_deref()).await {
        Ok(_) => serde_json::json!({ "status": "success", "message": format!("Removed {}", payload.package) }).into(),
        Err(e) => serde_json::json!({ "status": "error", "message": e.to_string() }).into(),
    }
}
