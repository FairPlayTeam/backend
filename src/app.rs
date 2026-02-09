use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_postgres::{Config, config::SslMode};

use crate::app::auth::{AuthError, AuthState, Claims, router};

mod auth;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub value: Arc<RwLock<f64>>,
    pub auth: Arc<AuthState>,
    pub hasher: Argon2<'static>,
    pub jwt_secret: String,
}

impl AppState {
    async fn new() -> Self {
        let mut db_cfg = Config::new();

        db_cfg
            .hostaddr(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            .ssl_mode(SslMode::Disable);

        db_cfg.user(
            std::env::var("POSTGRES_USER")
                .as_deref()
                .unwrap_or("fairplay-test"),
        );
        db_cfg.password(
            std::env::var("POSTGRES_PASSWORD")
                .as_deref()
                .unwrap_or("fairplay"),
        );
        db_cfg.dbname(
            std::env::var("POSTGRES_DB")
                .as_deref()
                .unwrap_or("fairplay-test"),
        );

        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("CRITICAL: JWT_SECRET environment variable must be set");

        let auth_state = AuthState::new(&db_cfg)
            .await
            .expect("Failed to connect to Postgres. Is it running?");

        Self {
            value: Arc::new(RwLock::new(0.0)),
            auth: Arc::new(auth_state),
            hasher: Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::DEFAULT),
            jwt_secret,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
struct Payload {
    value: f64,
}

async fn put_value(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Payload>,
) -> Result<(), AuthError> {
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
