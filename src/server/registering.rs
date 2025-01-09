use crate::server::app_state::AppState;

use axum::{
    debug_handler,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};

use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[allow(unused)]
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(register_bot))
        .with_state(state)
}

#[debug_handler]
#[allow(unused)]
async fn register_bot(
    State(state): State<AppState>,
    Query(payload): Query<RegisterBotPayload>,
) -> Result<(), RegisterBotError> {
    Ok(())
}

#[derive(Deserialize)]
#[allow(unused)]
struct RegisterBotPayload {
    name: String,
    port: u16,
}

#[derive(Error, Debug)]
#[allow(unused)]
enum RegisterBotError {
    #[error("a bot is already registered at this port")]
    PortTaken,
    #[error("a bot is already registered with this name")]
    NameTaken,
}

impl IntoResponse for RegisterBotError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
