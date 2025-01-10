use std::net::{Ipv4Addr, SocketAddr};

use tracing::error;

use match_server::server;

use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod cli;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    tracing_subscriber::fmt()
        // TODO: Add a argument to the cli tool to change the log level.
        .with_max_level(tracing::Level::TRACE)
        .init();

    let state = server::app_state::AppState::default();

    let address: SocketAddr = (Ipv4Addr::LOCALHOST, args.port).into();

    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            error!("Aborting, could not bind to address {address} due to following error: {error}");
            std::process::exit(1);
        }
    };

    let routes = Router::new()
        .with_state(state.clone())
        .nest("/login", server::login::routes(state.clone()))
        .fallback(|| async { "Invalid page" })
        .layer(TraceLayer::new_for_http());

    tokio::select! {
        _ = run_matches(state) => {},
        server_result = axum::serve(listener, routes) => {
            if let Err(error) = server_result {
                error!("Error whilst running server: {}", error);
            }
        }
    }
}

// #[derive(Error, Debug)]
// enum PlayError {
//     #[error("out of bounds")]
//     OutOfBounds,
//     #[error("no stone at given index")]
//     NoStoneAtIndex,
//     #[error("invalid json response: {0}")]
//     InvalidJsonResponse(#[from] reqwest::Error),
// }

async fn run_matches(_state: server::app_state::AppState) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
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
