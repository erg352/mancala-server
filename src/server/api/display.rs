use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::server::app_state::AppState;

#[debug_handler]
pub(super) async fn show_bots(
    State(state): State<AppState>,
) -> Result<Json<String>, ShowBotsError> {
    let connection = state.database.lock().await;

    let bots = connection
        .prepare("SELECT name, elo FROM bots ORDER BY elo DESC")?
        .query_map([], |row| {
            Ok(BotData {
                name: row.get(0)?,
                elo: row.get(1)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect();

    let payload = serde_json::to_string(&ShowBotsPayload { bots })?;

    Ok(Json(payload))
}

#[derive(Serialize)]
struct BotData {
    name: String,
    elo: u16,
}

#[derive(Serialize)]
struct ShowBotsPayload {
    bots: Vec<BotData>,
}

#[derive(Error, Debug)]
pub(super) enum ShowBotsError {
    #[error("could not serialize payload due to following error: {0}")]
    CouldNotSerialize(#[from] serde_json::Error),

    #[error("error whilst querying database: {0}")]
    DataBaseError(#[from] rusqlite::Error),
}

impl IntoResponse for ShowBotsError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::CouldNotSerialize(_error) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::DataBaseError(_error) => (StatusCode::INTERNAL_SERVER_ERROR, ""),
        }
        .into_response()
    }
}
