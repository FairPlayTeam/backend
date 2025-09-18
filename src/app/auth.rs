use std::collections::HashMap;

use axum::{Json, Router, extract::State, routing::post};
use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tokio_postgres::{Client, Config, NoTls};
use uuid::Uuid;

use super::AppState;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(#[serde_as(as = "Base64")] pub [u8; 32]);

#[derive(Debug)]
pub struct AuthState {
    db: Client,
    pub tokens: HashMap<Token, Uuid>,
}
impl AuthState {
    pub async fn new(cfg: &Config) -> Self {
        let (client, connection) = cfg.connect(NoTls)
            .await.unwrap();
        tokio::spawn(async {
            connection.await.unwrap(); // run the connection on a bg task
        });
        Self { db: client, tokens: Default::default() }
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
    let lock = state.auth.lock().await;
    let res = lock.db.execute(
        "WITH new_user AS (
            INSERT INTO public.users (email, password_hash)
            VALUES ($3, $2)
            RETURNING id
        )
        INSERT INTO public.user_accounts (id, username)
        SELECT id, $1
        FROM new_user;",
        &[&request.username, &state.base64.encode(request.secret), &request.email]).await;
    let res = res.map_err(|x| { Json(Err(x.to_string())) });
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
    let mut lock = state.auth.lock().await;
    let res = lock.db.query_one(
        "SELECT id, password_hash
        FROM public.users
        WHERE email = $1;",
        &[&request.email]).await;
    let res = res.map_err(|x| { Json(Err(x.to_string())) });
    let row = match res {
        Ok(row) => row,
        Err(err) => return err,
    };
    if state.base64.decode(row.get::<&str, String>("password_hash")).unwrap() == request.secret {
        let token = Token(rand::rng().random::<[u8; 32]>());
        assert!(lock.tokens.insert(token, row.get("id")).is_none()); // we should never see a token collision
        Json(Ok(token))
    } else {
        Json(Err("INVALID_CREDENTIALS".to_string()))
    }
}
