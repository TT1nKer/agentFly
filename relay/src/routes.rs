use axum::{Router, routing::get, response::Json, extract::WebSocketUpgrade, extract::Query};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db::RelayDb;
use crate::ws;

pub fn create_router(db: RelayDb) -> Router {
    let db = Arc::new(Mutex::new(db));

    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(db)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "agent-relay"
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
    state: axum::extract::State<Arc<Mutex<RelayDb>>>,
) -> impl axum::response::IntoResponse {
    let db = state.lock().await;
    db.register_device(&params.device_id, &params.device_type, &params.device_id).ok();
    drop(db);

    ws.on_upgrade(move |socket| ws::handle_ws(socket, params.device_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let db = crate::db::RelayDb::open(":memory:").unwrap();
        let app = create_router(db);

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
        assert_eq!(json["service"], "agent-relay");
    }
}
