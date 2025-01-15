use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{
    mancala::play_match::{play_match, Winner},
    server::app_state::{AppState, Bot},
};

pub async fn run_matches(state: AppState) {
    let current_bots: Arc<Mutex<Vec<Bot>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        // We iterate through all of the pending bots whilst removing them from the pending
        // bot array and have them match-make with each other bot.
        let mut pending_bots = state.pending_bots.lock().await;
        for bot in pending_bots.drain(..) {
            // We iterate through the whole list of current bots and setup the match between the
            // current pending bot and the already registered ones.
            let other_bots = current_bots.lock().await;
            for other_bot in other_bots.iter().cloned() {
                // We clone the bot and the state to let the async closure take their ownership.
                let bot = bot.clone();
                let state = state.clone();

                let addresses = [bot.address, other_bot.address];

                // Each match is run async, as the better part of the time taken to run a match
                // consists of HTTP communication and awaiting the bot's response.
                tokio::spawn(async move {
                    // We simply run the match between the two bots, and figure out what to do
                    // given it's outcome.
                    // Given a client is implemented as an Arc, cloning it is trivial and thus
                    // permetted here.
                    info!(
                        "Started match between {} and {}",
                        bot.name.clone(),
                        other_bot.name.clone()
                    );
                    match play_match(state.client.clone(), addresses).await {
                        Winner::Tie => handle_match_ending_tie(state, bot, other_bot).await,
                        Winner::ByDisqualification(bot_index, should_kick) => {
                            let bot = if bot_index == 0 { bot } else { other_bot };
                            handle_match_ending_disqualification(state, bot, should_kick).await
                        }
                        Winner::FairAndSquare(bot_index, delta) => {
                            let (winner, loser) = if bot_index == 0 {
                                (bot, other_bot)
                            } else {
                                (other_bot, bot)
                            };
                            handle_match_ending_fair_and_square(state, winner, loser, delta).await;
                        }
                    }
                });
            }

            // We add the new bot after spawning all tokio tasks so as to not have the chance of
            // a bot fighting against itself :p.
            current_bots.lock().await.push(bot);
        }
    }
}

/// Handles what should happen when two bots play a match and one wins by a certain points delta.
async fn handle_match_ending_fair_and_square(state: AppState, winner: Bot, loser: Bot, delta: u8) {
    let (winner_elo, loser_elo) = (
        winner.elo.saturating_add(delta as u16),
        loser.elo.saturating_sub(delta as u16),
    );

    let connection = state.database.lock().await;

    let handle_database_output = |result: rusqlite::Result<usize>| match result {
        Ok(updated_row_count) if updated_row_count != 1 => {
            error!(
                "When changing the elo of a match's winner, {} rows where updated instead of 1",
                updated_row_count
            );
        }
        Err(error) => {
            error!("Error encountered when updating player's elo: {}", error);
        }
        _ => {}
    };

    handle_database_output(connection.execute(
        "UPDATE bots SET elo = ?1 WHERE id = ?2",
        (winner_elo, winner.id),
    ));

    handle_database_output(connection.execute(
        "UPDATE bots SET elo = ?1 WHERE id = ?2",
        (loser_elo, loser.id),
    ));
}

/// Handles what should be done when too bots play a match and end up scoring the
/// same amount.
async fn handle_match_ending_tie(_state: AppState, _bot_a: Bot, _bot_b: Bot) {
    // TODO: Figure out how to vary each player's elo in the case of a tie.
}

/// Handles what should happen when a bot loses by disqualification
async fn handle_match_ending_disqualification(
    _state: AppState,
    _disqualified_bot: Bot,
    should_kick: bool,
) {
    if !should_kick {
        return;
    }

    warn!("A bot should be kicked, but wasn't due to the codebase being WIP.");
}
