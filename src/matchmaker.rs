use std::time::Duration;

use tracing::{error, trace, warn};

use crate::{
    mancala::play_match::{play_match, Winner},
    server::app_state::{AppState, Bot},
};

pub async fn run_matches(state: AppState) {
    trace!("Started the matchmaking task.");

    loop {
        // We iterate through all of the pending bots whilst removing them from the pending
        // bot array and have them match-make with each other bot.

        // We are dropping the lock to the pending bots early to avoid complications. (the cost
        // of the copy can be considered minimal given the small (if not 0) size of the
        // pending_bots array).
        let pending_bots: Vec<_> = {
            let mut lock = state.pending_bots.lock().await;
            lock.drain(..).collect()
        };

        // We don't need to allocate any memory if there are no bots that are pending, so we don't
        let other_bots = if pending_bots.is_empty() {
            Vec::new()
        } else {
            state.connected_bots.lock().await.iter().cloned().collect()
        };

        for bot_a in pending_bots {
            // We iterate through the whole list of current bots and setup the match between the
            // current pending bot and the already registered ones.
            for bot_b in other_bots.iter() {
                // We clone the bot and the state to let the async closure take their ownership.
                let state = state.clone();

                // Each match is run async, as the better part of the time taken to run a match
                // consists of HTTP communication and awaiting the bot's response.
                // We are also running two matches, one where the first player is bot_a and one
                // where the first player is bot_b for added fairness.
                tokio::spawn(launch_match(state.clone(), bot_a.clone(), bot_b.clone()));
                tokio::spawn(launch_match(state.clone(), bot_b.clone(), bot_a.clone()));
            }

            // We add the new bot after spawning all tokio tasks so as to not have the chance of
            // a bot fighting against itself :p.
            state.connected_bots.lock().await.insert(bot_a.clone());
        }
        // Sleep for some time, as there is no need to run this code ad-nauseum given bots won't
        // connect frequently (and even if they do, them waiting a second for their matches to
        // start isn't the end of the world).
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn launch_match(state: AppState, bot_a: Bot, bot_b: Bot) {
    trace!(
        "Started match between {} (player 1) and {} (player 2)",
        bot_a.name.clone(),
        bot_b.name.clone()
    );
    match play_match(state.client.clone(), [bot_a.address, bot_b.address]).await {
        Winner::Tie => handle_match_ending_tie(state, bot_a, bot_b).await,
        Winner::ByDisqualification(bot_index, should_kick) => {
            let bot = if bot_index == 0 { bot_a } else { bot_b };
            handle_match_ending_disqualification(state, bot, should_kick).await
        }
        Winner::FairAndSquare(bot_index, delta) => {
            let (winner, loser) = if bot_index == 0 {
                (bot_a, bot_b)
            } else {
                (bot_b, bot_a)
            };
            handle_match_ending_fair_and_square(state, winner, loser, delta).await;
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
