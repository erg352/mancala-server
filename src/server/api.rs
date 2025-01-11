use crate::server::app_state::AppState;

use axum::{routing::get, Router};

mod display;
mod login;
mod register;

#[allow(unused)]
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register", get(register::register_bot))
        .route("/login", get(login::login))
        .route("/display", get(display::show_bots))
        .with_state(state)
}
