use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
    routing::post,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tokio_postgres::Config;
use uuid::Uuid;

pub mod db;
use super::AppState;
use crate::app::auth::db::Database;

const EXPIRATION_SECONDS: u64 = 86400; // 24 hours

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug)]
pub struct AuthState {
    pub db: Database,
}

impl AuthState {
    pub async fn new(cfg: &Config) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            db: Database::new(cfg).await?,
        })
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    UserExists,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed")
            }
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::UserExists => (StatusCode::CONFLICT, "User already exists"),
        };
        (status, body).into_response()
    }
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AuthError::MissingCredentials)?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AuthError::MissingCredentials)?;
        if !auth_str.starts_with("Bearer ") {
            return Err(AuthError::MissingCredentials);
        }
        let token = &auth_str[7..];

        // Use cached secret from state, never env vars in hot path
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
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
) -> Result<(), AuthError> {
    let salt = SaltString::generate(&mut OsRng);

    // Explicit error mapping instead of unwrap
    let password_str =
        std::str::from_utf8(&request.secret).map_err(|_| AuthError::TokenCreation)?;

    let hash = state
        .hasher
        .hash_password(password_str.as_bytes(), &salt)
        .map_err(|_| AuthError::TokenCreation)?;

    state
        .auth
        .db
        .create_user(&request.username, &request.email, hash.serialize())
        .await
        .map_err(|_| AuthError::UserExists)?;

    Ok(())
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    #[serde_as(as = "Base64")]
    secret: Vec<u8>,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let row = state
        .auth
        .db
        .get_user(&request.email)
        .await
        .map_err(|_| AuthError::WrongCredentials)?;

    let password_hash_str = row.get::<_, &str>("password_hash");
    let parsed_hash =
        PasswordHash::new(password_hash_str).map_err(|_| AuthError::WrongCredentials)?;

    let password_str =
        std::str::from_utf8(&request.secret).map_err(|_| AuthError::WrongCredentials)?;

    state
        .hasher
        .verify_password(password_str.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::WrongCredentials)?;

    let user_id: Uuid = row.get("id");

    // Handle time error explicitly, though extremely rare
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::TokenCreation)?
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: now + (EXPIRATION_SECONDS as usize),
    };

    // Use cached secret from state
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(LoginResponse { token }))
}
