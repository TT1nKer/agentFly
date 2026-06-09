use axum::{Router, routing::get, response::Json, extract::WebSocketUpgrade, extract::Query};
use serde_json::json;
use std::sync::Arc;
use crate::ws;
use crate::ws::RelayState;

pub fn create_router() -> Router {
    let state = Arc::new(RelayState::new());

    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .route("/devices", get(devices_handler))
        .with_state(state)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "agent-relay"
    }))
}

async fn devices_handler(
    state: axum::extract::State<Arc<RelayState>>,
) -> Json<serde_json::Value> {
    let devices = state.list_devices().await;
    Json(json!({
        "devices": devices,
        "count": devices.len()
    }))
}

#[derive(serde::Deserialize)]
struct WsParams {
    device_id: String,
    device_type: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    state: axum::extract::State<Arc<RelayState>>,
) -> impl axum::response::IntoResponse {
    let device_id = params.device_id;
    let device_type = params.device_type;

    let state_inner = state.0.clone();
    ws.on_upgrade(move |socket| ws::handle_ws(socket, device_id, device_type, state_inner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_devices_endpoint() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
