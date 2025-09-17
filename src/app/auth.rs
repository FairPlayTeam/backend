use std::collections::HashMap;

use axum::{Json, Router, extract::State, routing::post};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use super::AppState;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(#[serde_as(as = "Base64")] pub [u8; 32]);

#[derive(Debug, Default, Clone)]
pub struct AuthState {
    db: HashMap<String, Vec<u8>>,
    pub tokens: HashMap<Token, String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    #[serde_as(as = "Base64")]
    secret: Vec<u8>,
}
async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Json<Result<(), &'static str>> {
    let mut lock = state.auth.lock().await;
    if lock.db.contains_key(&request.username) {
        return Json(Err("ALREADY_EXISTS"));
    }
    lock.db.insert(request.username, request.secret);
    Json(Ok(()))
}

type LoginRequest = RegisterRequest;
async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Json<Result<Token, &'static str>> {
    let mut lock = state.auth.lock().await;
    match lock.db.get(&request.username) {
        Some(x) if *x == request.secret => {
            let token = Token(rand::rng().random::<[u8; 32]>());
            assert!(lock.tokens.insert(token, request.username).is_none()); // we should never see a token collision
            Json(Ok(token))
        }
        None | Some(_) => Json(Err("INVALID_CREDENTIALS")),
    }
}
