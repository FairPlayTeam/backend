use std::{net::{IpAddr, Ipv4Addr}, sync::Arc};

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio_postgres::{config::SslMode, Config};

use crate::app::auth::{AuthState, Token, router};

mod auth;

#[derive(Debug, Clone)]
struct AppState {
    value: Arc<RwLock<f64>>,
    auth: Arc<Mutex<AuthState>>,
}
impl AppState {
    async fn new() -> Self {
        let mut cfg = Config::new();

        cfg
            .hostaddr(IpAddr::V4(Ipv4Addr::LOCALHOST))
            .ssl_mode(SslMode::Disable);

        if let Ok(user) = dotenvy::var("POSTGRES_USER") {
            cfg.user(user);
        } else {
            cfg.user("fairplay-test");
        }
        if let Ok(password) = dotenvy::var("POSTGRES_PASSWORD") {
            cfg.password(password);
        } else {
            cfg.password("fairplay");
        }
        cfg.dbname("fairplay-test");

        Self {
            value: Arc::new(RwLock::new(0.0)),
            auth: Arc::new(Mutex::new(AuthState::new(&cfg).await)),
        }
    }
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

pub async fn new_app() -> Router {
    Router::new()
        .route("/value", post(put_value).get(get_value))
        .nest("/auth", router())
        .with_state(AppState::new().await)
}
