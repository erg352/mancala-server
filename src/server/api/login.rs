use std::net::SocketAddr;

use crate::server::app_state::{AppState, Bot};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    debug_handler,
    extract::{ConnectInfo, Query, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use thiserror::Error;
use tracing::warn;

#[debug_handler]
pub(super) async fn login(
    State(state): State<AppState>,
    Query(payload): Query<LoginBotPayload>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<(), LoginBotError> {
    let connection = state.database.lock().await;

    let (bot, hashed_password): (Bot, String) = connection
        .query_row(
            "SELECT id, name, elo, password FROM bots WHERE name = ?1",
            params![payload.name],
            |row| {
                Ok((
                    Bot {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        elo: row.get(2)?,
                        address,
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
        .ok()
        .is_none()
    {
        return Err(LoginBotError::InvalidPassword);
    }

    if state.pending_bots.lock().await.contains(&bot)
        || state.connected_bots.lock().await.contains(&bot)
    {
        warn!(
            "Bot {} attempted to login whilst already being logged in.",
            bot.name
        );
        return Ok(());
    }

    state.pending_bots.lock().await.push(bot);

    Ok(())
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
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::HasherError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "password is corrupted"),
        }
        .into_response()
    }
}
