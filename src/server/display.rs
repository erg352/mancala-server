use axum::{debug_handler, extract::State, routing::get, Json, Router};

use super::app_state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/show_bots", get(show_bots))
        .with_state(state)
}

#[debug_handler]
async fn show_bots(State(state): State<AppState>) -> Json<String> {
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
