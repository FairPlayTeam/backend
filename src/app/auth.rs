use std::collections::HashMap;

use argon2::{
    PasswordVerifier,
    password_hash::{PasswordHashString, PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, Router, extract::State, routing::post};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tokio::sync::Mutex;
use tokio_postgres::Config;
use uuid::Uuid;

use crate::app::auth::db::Database;

use super::AppState;

mod db;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(#[serde_as(as = "Base64")] pub [u8; 32]);

#[derive(Debug)]
pub struct AuthState {
    db: Database,
    pub tokens: Mutex<HashMap<Token, Uuid>>,
}
impl AuthState {
    pub async fn new(cfg: &Config) -> Self {
        Self {
            db: Database::new(cfg).await,
            tokens: Default::default(),
        }
    }
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
    email: String,
    #[serde_as(as = "Base64")]
    secret: Vec<u8>,
}
async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Json<Result<(), String>> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = state.hasher.hash_password(&request.secret, &salt).unwrap();

    let res = state
        .auth
        .db
        .create_user(&request.username, &request.email, hash.serialize())
        .await;
    let res = res.map_err(|x| Json(Err(x.to_string())));
    if let Err(e) = res {
        return e;
    }
    Json(Ok(()))
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    #[serde_as(as = "Base64")]
    secret: Vec<u8>,
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Json<Result<Token, String>> {
    let res = state.auth.db.get_user(&request.email).await;
    let res = res.map_err(|x| Json(Err(x.to_string())));
    let row = match res {
        Ok(row) => row,
        Err(err) => return err,
    };

    if state
        .hasher
        .verify_password(
            &request.secret,
            &PasswordHashString::new(row.get::<_, &str>("password_hash"))
                .unwrap()
                .password_hash(),
        )
        .is_ok()
    {
        let token = Token(rand::rng().random::<[u8; 32]>());
        assert!(
            state
                .auth
                .tokens
                .lock()
                .await
                .insert(token, row.get("id"))
                .is_none()
        ); // we should never see a token collision
        Json(Ok(token))
    } else {
        Json(Err("INVALID_CREDENTIALS".to_string()))
    }
}
