use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::app::auth::{AuthState, Token, router};

mod auth;

#[derive(Debug, Default, Clone)]
struct AppState {
    value: Arc<RwLock<f64>>,
    auth: Arc<Mutex<AuthState>>,
}
impl AppState {
    async fn validate_token(&self, token: &Token) -> bool {
        self.auth.lock().await.tokens.contains_key(token)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
struct Payload {
    token: Token,
    value: f64,
}

async fn put_value(
    State(state): State<AppState>,
    Json(payload): Json<Payload>,
) -> Result<(), &'static str> {
    // this is protected and needs a token
    if !state.validate_token(&payload.token).await {
        return Err("INVALID_TOKEN");
    }
    *state.value.write().await = payload.value;
    Ok(())
}
async fn get_value(State(state): State<AppState>) -> Json<f64> {
    Json(*state.value.read().await)
}

pub fn new_app() -> Router {
    Router::new()
        .route("/value", post(put_value).get(get_value))
        .nest("/auth", router())
        .with_state(AppState::default())
}
