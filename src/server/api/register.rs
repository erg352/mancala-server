use crate::server::app_state::AppState;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    debug_handler,
    extract::{Query, State},
    response::IntoResponse,
};

use reqwest::StatusCode;
use rusqlite::params;
use serde::Deserialize;
use thiserror::Error;

#[debug_handler]
pub(super) async fn register_bot(
    State(state): State<AppState>,
    Query(payload): Query<RegisterBotPayload>,
) -> Result<(), RegisterBotError> {
    let connection = state.database.lock().await;

    let column_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM bots WHERE name = ?1",
        params![payload.name],
        |row| row.get(0),
    )?;

    if column_count != 0 {
        return Err(RegisterBotError::NameInUse);
    }

    let password = payload.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed_password = argon2.hash_password(password, &salt)?.to_string();

    connection.execute(
        "INSERT INTO bots (name, password, elo) VALUES (?1, ?2, 1000)",
        params![payload.name, hashed_password],
    )?;

    Ok(())
}

#[derive(Deserialize)]
pub(super) struct RegisterBotPayload {
    name: String,
    password: String,
}

#[derive(Error, Debug)]
pub(super) enum RegisterBotError {
    #[error("name is already registed.")]
    NameInUse,

    #[error("rusqlite error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("argon2 error: {0}")]
    HasherError(argon2::password_hash::Error),
}

// Needed because argon2::password_has::Error doesn't implement std::error::Error 😤
impl From<argon2::password_hash::Error> for RegisterBotError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HasherError(value)
    }
}

impl IntoResponse for RegisterBotError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NameInUse => (StatusCode::UNAUTHORIZED, "name is already taken"),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::HasherError(_) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
        }
        .into_response()
    }
}
