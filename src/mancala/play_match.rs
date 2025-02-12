use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::error;

use super::{Board, Game};

/// Plays an entire match of mancala between two players, and returns information about
/// who won (if anyone won), in what manner and by how much.
pub async fn play_match(players: impl Into<[Arc<Mutex<WebSocket>>; 2]>) -> Winner {
    let players = players.into();

    let mut game = Game::default();
    let mut current_player = 0;

    while !game.is_finished() {
        const CONNECTION_RETRY_COUNT: u8 = 8;
        const QUERY_RETRY_COUNT: u8 = 2;

        let mut connection_retries = CONNECTION_RETRY_COUNT;
        let mut querying_retries = QUERY_RETRY_COUNT;
        let player_move = loop {
            match game
                .send_to_player(current_player, players[current_player].clone())
                .await
            {
                Ok(response) => {
                    let response = response.value;
                    if game.is_move_valid(current_player as u8, response) {
                        break response;
                    }
                    querying_retries -= 1;
                }

                // Err(PlayerResponseError::RequestError(e)) => {
                //     trace!("Request Error: {e}");
                //
                //     connection_retries -= 1;
                //     if connection_retries == 0 {
                //         return Winner::ByDisqualification(1 - current_player as u8, true);
                //     }
                // }
                Err(PlayerResponseError::InvalidResponse) => {
                    // We managed to connect to the player, so might as well give
                    // them the benefit of the doubt
                    connection_retries = QUERY_RETRY_COUNT;

                    querying_retries -= 1;

                    if querying_retries == 0 {
                        return Winner::ByDisqualification(1 - current_player as u8, false);
                    }
                }

                Err(PlayerResponseError::CouldNotSerialize(error)) => {
                    error!("Could not serialize the the board to send it to the player due to following error: \"{error}\", aborting instead and resoliving match in a tie.");
                    return Winner::Tie;
                }

                Err(PlayerResponseError::SendFailed(_))
                | Err(PlayerResponseError::ReceiveFailed(_))
                | Err(PlayerResponseError::DidNotReceiveResponse) => {
                    connection_retries -= 1;
                    if connection_retries == 0 {
                        return Winner::ByDisqualification(1 - current_player as u8, true);
                    }
                }
            }
        };

        current_player = game.play(current_player, player_move as usize);
    }

    if game.points[0] == game.points[1] {
        Winner::Tie
    } else {
        let delta = (game.points[0] as i8 - game.points[1] as i8).unsigned_abs();
        Winner::FairAndSquare(
            if game.points[0] > game.points[1] {
                0
            } else {
                1
            },
            delta,
        )
    }
}

/// Summarizes the end of a mancala match between two bots.
pub enum Winner {
    /// One of the bots was unable to communicate with the server either
    /// because it has disconnected, or because it was unable to send back
    /// appropriate data. Thus the other bot won by disqualification.
    ByDisqualification(u8, bool),

    /// Both bots played correctly until the end of the game, but one played
    /// better than the other. The first paramter describes which player won
    /// (the first or the second) and the second paramter describles by how much
    /// (the difference of score between the winner and the loser in absolute value).
    FairAndSquare(u8, u8),

    /// Both bots played correctly until the end of the game, but when tallying up the
    /// scores, they both had the same amount of points and thus the game became a tie.
    Tie,
}

impl Game {
    fn to_json(&self, player: usize) -> Result<String, PlayerResponseError> {
        debug_assert!(player < 2);

        #[derive(Serialize)]
        struct SerializableGame {
            boards: [Board; 2],
            points: [u8; 2],
        }

        Ok(serde_json::to_string(&SerializableGame {
            boards: [self.boards[player], self.boards[1 - player]],
            points: [self.points[player], self.points[1 - player]],
        })?)
    }

    async fn send_to_player(
        &self,
        player: usize,
        socket: Arc<Mutex<WebSocket>>,
    ) -> Result<PlayerResponse, PlayerResponseError> {
        debug_assert!(player < 2);

        let serialized = self.to_json(player)?;

        let mut socket = socket.lock().await;

        socket
            .send(serialized.into())
            .await
            .map_err(|e| PlayerResponseError::SendFailed(e.into()))?;

        let response = socket
            .recv()
            .await
            .map(|e| e.map_err(|e| PlayerResponseError::ReceiveFailed(e.into())))
            .ok_or(PlayerResponseError::DidNotReceiveResponse)??;

        Ok(match response {
            Message::Text(text) => serde_json::from_str::<PlayerResponse>(&text),
            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {
                return Err(PlayerResponseError::InvalidResponse)
            }
            Message::Close(_close_frame) => todo!("handle client closing the web socket"),
        }?)
    }
}

#[derive(Deserialize)]
pub struct PlayerResponse {
    pub value: u8,
}

#[derive(Error, Debug)]
enum PlayerResponseError {
    #[error("could not send information to the player due to following error: {0}")]
    SendFailed(Box<dyn std::error::Error>),

    #[error("failed to retreive data from player due to following error: {0}")]
    ReceiveFailed(Box<dyn std::error::Error>),

    #[error("did not receive a response from the player")]
    DidNotReceiveResponse,

    #[error("invalid response from player")]
    InvalidResponse,

    #[error("failed to serialize board due to error: {0}")]
    CouldNotSerialize(#[from] serde_json::Error),
}
