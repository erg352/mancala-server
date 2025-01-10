use std::net::SocketAddr;

use crate::server::app_state::AppState;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    debug_handler,
    extract::{ConnectInfo, Query, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use thiserror::Error;

#[debug_handler]
pub(super) async fn login(
    State(state): State<AppState>,
    Query(payload): Query<LoginBotPayload>,
    ConnectInfo(_address): ConnectInfo<SocketAddr>,
) -> Result<(), LoginBotError> {
    let connection = state.database.lock().unwrap();

    let query: Option<String> = connection
        .query_row(
            "SELECT password, FROM bots WHERE name = ?1",
            params![payload.name],
            |row| row.get(0),
        )
        .optional()?;

    match query {
        None => Err(LoginBotError::InvalidName),

        Some(hashed_password) => {
            let parsed_hash = PasswordHash::new(&hashed_password)?;
            if Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .ok()
                .is_none()
            {
                return Err(LoginBotError::InvalidPassword);
            }

            Ok(())
        }
    }
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
        let body = match self {
            Self::InvalidName => "bot is not registered",
            Self::InvalidPassword => "password is incorrect",
            Self::DatabaseError(_) => "",
            Self::HasherError(_) => "password is corrupted",
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
