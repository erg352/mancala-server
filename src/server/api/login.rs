use std::sync::Arc;

use crate::server::app_state::{AppState, Bot};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    debug_handler,
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use rand::{distr::StandardUniform, rngs::StdRng, Rng, SeedableRng};
use reqwest::StatusCode;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::error;

type Token = String;

#[derive(Serialize, Deserialize)]
struct LoginResponse<'a> {
    name: &'a str,
}

#[debug_handler]
pub(super) async fn login(
    State(state): State<AppState>,
    Query(payload): Query<LoginBotPayload>,
    web_socket: WebSocketUpgrade,
) -> Result<Token, LoginBotError> {
    let connection = state.database.lock().await;

    let secret: Arc<[u8]> = StdRng::from_os_rng()
        .sample_iter::<u8, _>(&StandardUniform)
        .take(32)
        .collect();

    let (mut bot, hashed_password): (Bot, String) = connection
        .query_row(
            "SELECT id, name, elo, password FROM bots WHERE name = ?1",
            params![payload.name],
            |row| {
                Ok((
                    Bot {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        elo: row.get(2)?,
                        socket: None,
                        secret,
                    },
                    row.get(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(LoginBotError::InvalidName)?;

    let parsed_hash = PasswordHash::new(&hashed_password)?;
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(LoginBotError::InvalidPassword);
    }

    if state.pending_bots.lock().await.contains(&bot)
        || state.connected_bots.lock().await.contains(&bot)
    {
        return Err(LoginBotError::AlreadyLoggedIn);
    }

    let response = LoginResponse { name: &bot.name };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &response,
        &jsonwebtoken::EncodingKey::from_secret(&bot.secret),
    );

    let token = token.map_err(|_| LoginBotError::CouldNotEncodeToken)?;

    let result = web_socket.on_upgrade(|socket| async move {
        bot.socket = Some(Arc::new(Mutex::new(socket)));
        state.pending_bots.lock().await.push(bot);
    });

    error!("the result was: {result:?}");

    Ok(token)
}

#[derive(Deserialize)]
pub(super) struct LoginBotPayload {
    name: String,
    password: String,
}

#[derive(Error, Debug)]
pub(super) enum LoginBotError {
    #[error("rusqlite error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("name is not in the database")]
    InvalidName,

    #[error("password is incorrect")]
    InvalidPassword,

    #[error("argon2 error: {0}")]
    HasherError(argon2::password_hash::Error),

    #[error("bot already logged in.")]
    AlreadyLoggedIn,

    #[error("could not encode token")]
    CouldNotEncodeToken,
}

// Needed because argon2::password_has::Error doesn't implement std::error::Error 😤
impl From<argon2::password_hash::Error> for LoginBotError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HasherError(value)
    }
}

impl IntoResponse for LoginBotError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidName => (StatusCode::UNAUTHORIZED, "invalid username"),
            Self::InvalidPassword => (StatusCode::UNAUTHORIZED, "invalid password"),
            Self::AlreadyLoggedIn => (StatusCode::UNAUTHORIZED, "already logged in"),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::HasherError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "password is corrupted"),
            Self::CouldNotEncodeToken => (StatusCode::INTERNAL_SERVER_ERROR, ""),
        }
        .into_response()
    }
}
