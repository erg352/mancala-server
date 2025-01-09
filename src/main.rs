#![allow(unused)]

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::trace::TraceLayer;

mod cli;

#[derive(Clone, Default)]
struct AppState {}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let address = std::net::Ipv4Addr::LOCALHOST;
    let port = args.port;
    let listener = TcpListener::bind((address, port)).await.unwrap();

    let state = AppState::default();

    let routes = Router::new()
        .route("/register", get(register_bot))
        .fallback(|| async { "Invalid page" })
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, routes).await.unwrap();
}

#[derive(Deserialize)]
struct RegisterBotPayload {
    name: String,
    port: u16,
}

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
enum PlayError {
    #[error("out of bounds")]
    OutOfBounds,
    #[error("no stone at given index")]
    NoStoneAtIndex,
    #[error("invalid json response: {0}")]
    InvalidJsonResponse(#[from] reqwest::Error),
}

#[debug_handler]
async fn register_bot(
    State(state): State<AppState>,
    Query(payload): Query<RegisterBotPayload>,
) -> Result<(), RegisterBotError> {
    Ok(())
}

/*
async fn play_turn(
    client: Client,
    game_container: Arc<RwLock<Game>>,
    player_id: u8,
) -> Result<(), PlayError> {
    assert!(player_id <= 1);

    #[derive(Serialize)]
    struct SentPayload {
        board: [[u8; 6]; 2],
        scores: [u8; 2],
    }

    #[derive(Deserialize)]
    struct ReceivedPayload {
        #[serde(rename = "move")]
        attempted_move: u8,
    }

    let game = game_container.read().await;

    let board = [
        game.board[player_id as usize],
        game.board[1 - player_id as usize],
    ];

    let scores = [
        game.scores[player_id as usize],
        game.scores[1 - player_id as usize],
    ];

    let payload = serde_json::to_string(&SentPayload { board, scores }).unwrap();

    let response = client
        .get(format!("localhost:{}/play", game.ports[player_id as usize]))
        .body(payload)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    let (board_index, stone_index) = match response.json::<ReceivedPayload>().await {
        Err(error) => return Err(PlayError::InvalidJsonResponse(error)),
        Ok(payload) => {
            let player_board_empty = board[0].iter().all(|piece| *piece == 0);

            let stone_index = payload
                .attempted_move
                .wrapping_sub(player_board_empty as u8 * 6) as usize;

            let board_index = player_board_empty as u8 as usize;

            if stone_index >= 6 {
                return Err(PlayError::OutOfBounds);
            }

            if board[board_index][stone_index] == 0 {
                return Err(PlayError::NoStoneAtIndex);
            }

            (board_index, stone_index)
        }
    };

    Ok(())
}
*/
