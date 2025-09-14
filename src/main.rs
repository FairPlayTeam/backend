use std::{net::Ipv4Addr, sync::Arc};

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::RwLock};

#[derive(Debug, Default, Clone)]
struct AppState {
    value: Arc<RwLock<f64>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
struct Payload {
    value: f64,
}

#[tokio::main]
async fn main() {
    let value = Arc::new(RwLock::const_new(0.0));

    let app = Router::new()
        .route("/value", post(put_value).get(get_value))
        .with_state(AppState {
            value: value.clone(),
        });

    axum::serve(
        TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), 8080))
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap()
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
