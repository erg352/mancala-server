use axum::{debug_handler, extract::State, Json};

use crate::server::app_state::AppState;

#[debug_handler]
pub(super) async fn show_bots(State(state): State<AppState>) -> Json<String> {
    let connection = state.database.lock().unwrap();

    let mut query = connection
        .prepare("SELECT name, elo FROM bots ORDER BY elo DESC")
        .unwrap();

    let bots: Vec<(String, usize)> = query
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .filter_map(|row| row.ok())
        .collect();

    Json(serde_json::to_string(&bots).unwrap())
}
