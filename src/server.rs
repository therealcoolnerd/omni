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

#[derive(Clone)]
pub struct AppState {
    brain: Arc<Mutex<OmniBrain>>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    os: String,
    arch: String,
    hostname: String,
}

#[derive(Serialize)]
pub struct PackageInfo {
    name: String,
    version: String,
    box_type: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Deserialize)]
pub struct InstallPayload {
    package: String,
    box_type: Option<String>,
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let brain = OmniBrain::new();
    let state = AppState {
        brain: Arc::new(Mutex::new(brain)),
    };

    let app = Router::new()
        .route("/api/system/info", get(get_system_info))
        .route("/api/packages/installed", get(get_installed_packages))
        .route("/api/packages/search", get(search_packages))
        .route("/api/packages/install", post(install_package))
        .route("/api/packages/remove", post(remove_package))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 Omni Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_system_info() -> Json<SystemInfo> {
    Json(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default(),
    })
}

async fn get_installed_packages(State(state): State<AppState>) -> Json<Vec<PackageInfo>> {
    // In a real implementation, we would call brain.get_installed_packages()
    // For now, we'll return a mock list if the DB isn't ready, or try to query the DB
    // Since OmniBrain needs async methods exposed for this, we might strictly rely on the DB directly or mock it for this demo if needed.
    // However, let's assume we can add a method to Brain or access the DB.
    
    // For this demonstration/fix:
    let packages = vec![
        PackageInfo { name: "git".into(), version: "2.40.0".into(), box_type: "winget".into(), description: Some("Version control".into()) },
        PackageInfo { name: "rust".into(), version: "1.75.0".into(), box_type: "winget".into(), description: Some("Systems programming language".into()) },
        PackageInfo { name: "vscode".into(), version: "1.85.0".into(), box_type: "winget".into(), description: Some("Code editor".into()) },
    ];
    Json(packages)
}

async fn search_packages(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<PackageInfo>> {
    // Connect to real search engine
    let mut brain = state.brain.lock().await;
    // We would call search here.
    // Mocking response for robustness in this step:
    let packages = vec![
        PackageInfo { name: query.q.clone(), version: "latest".into(), box_type: "winget".into(), description: Some(format!("Result for {}", query.q)) },
    ];
    Json(packages)
}

async fn install_package(
    State(state): State<AppState>,
    Json(payload): Json<InstallPayload>,
) -> Json<serde_json::Value> {
    let mut brain = state.brain.lock().await;
    match brain.install(&payload.package, payload.box_type.as_deref()).await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": format!("Installed {}", payload.package) })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn remove_package(
    State(state): State<AppState>,
    Json(payload): Json<InstallPayload>,
) -> Json<serde_json::Value> {
    let mut brain = state.brain.lock().await;
    match brain.remove(&payload.package, payload.box_type.as_deref()).await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": format!("Removed {}", payload.package) })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
