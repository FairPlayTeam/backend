use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
struct AppState {
    value: Arc<RwLock<f64>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
struct Payload {
    value: f64,
}

async fn put_value(
    State(AppState { value: state }): State<AppState>,
    Json(payload): Json<Payload>,
) {
    *state.write().await = payload.value;
}
async fn get_value(State(state): State<AppState>) -> Json<f64> {
    Json(*state.value.read().await)
}

pub fn new_app() -> Router {
    let value = Arc::new(RwLock::const_new(0.0));

    Router::new()
        .route("/value", post(put_value).get(get_value))
        .with_state(AppState { value })
}
