use crate::server::app_state::AppState;

use axum::{routing::get, Router};

mod login_handler;
mod register_handler;

#[allow(unused)]
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register", get(register_handler::register_bot))
        .route("/", get(login_handler::login))
        .with_state(state)
}

// let password: Option<String> = connection
//     .query_row(
//         "SELECT password FROM bots WHERE name = ?1",
//         params![payload.name],
//         |row| row.get(0),
//     )
//     .optional()
//     .unwrap();
//
// if password != payload.pass
