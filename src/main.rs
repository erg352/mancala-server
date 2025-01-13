use std::net::{Ipv4Addr, SocketAddr};

use match_server::server::{self, app_state::AppState};

use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use color_eyre::{eyre::WrapErr, Result as EyreResult};

mod cli;

#[tokio::main]
async fn main() -> EyreResult<()> {
    let args = cli::Args::parse();

    color_eyre::install()?;
    tracing_subscriber::fmt().with_max_level(args.log).init();

    let state = AppState::default();

    let address = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), args.port);

    let listener = TcpListener::bind(address)
        .await
        .wrap_err_with(|| format!("Could not bind the server to address {address}"))?;

    let routes = Router::new()
        .with_state(state.clone())
        .nest("/api/", server::api::routes(state.clone()))
        .layer(TraceLayer::new_for_http());

    tokio::select! {
        _ = run_matches(state) => {},
        result = axum::serve(listener, routes.into_make_service_with_connect_info::<SocketAddr>()) => result?
    }

    Ok(())
}

async fn run_matches(_state: AppState) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
