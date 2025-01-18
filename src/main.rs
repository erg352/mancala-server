use std::net::{Ipv4Addr, SocketAddr};

use match_server::{
    matchmaker::run_matches,
    server::{self, app_state::AppState},
};

use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use color_eyre::{eyre::WrapErr, Result as EyreResult};

mod cli;

#[tokio::main]
async fn main() -> EyreResult<()> {
    // Parse the command line arguments.
    let args = cli::Args::parse();

    // Setup the error handling crates (does not matter if it is done before or after
    // parsing the command line arguments as clap handles its errors itself, so might as well
    // not setup these crates should an error accur whilst parsing arguments as that would just be
    // a waste of time).
    color_eyre::install()?;
    tracing_subscriber::fmt().with_max_level(args.log).init();

    // The app state contains all of the data for the application. It is trivialy cloneable,
    // as all of it's data is in Arcs or other smart pointers. This cloneability is needed for
    // axum and the matchmaker.
    let state = AppState::new(&args.database);

    // We are using TCP instead of UDP even if we consider the network to be reliable (and in the
    // offchance it isn't, there should be enough guardrails to prevent undesireable behavior)
    // because axum works better with TCP than it does with UDP. Regardless, the messages sent
    // are fairly small, and overall this **should** not be a bottleneck.
    let address = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), args.port);

    let listener = TcpListener::bind(address)
        .await
        .wrap_err_with(|| format!("Could not bind the server to address {address}"))?;

    // Create all the routes for the server.
    let routes = Router::new()
        // The state must be placed AFTER all the current router's routes but BEFORE the nested
        // router's routes, as those may have a completely different state.
        .with_state(state.clone())
        // This is a different router, so we put it after the with_state call, but we still want to
        // pass in the app state.
        .nest("/api/", server::api::routes(state.clone()))
        .fallback_service(tower_http::services::ServeDir::new(args.static_routes))
        // The trace layer should be applied to all routes from root to the nested routes, hence
        // it's pace after all routes have been declared. (middle ware is applied from bottom to
        // top)
        .layer(TraceLayer::new_for_http());

    // Whichever async function returns first will force the other to do so too. In theory, neither
    // one of these functions should ever return, but in the off chance one does, we can assume it
    // to be because of some error, and given all branches depend on one another, the end of one
    // branch should result in the end of all branches.
    tokio::select! {
        _ = run_matches(state) => {},
        result = axum::serve(listener, routes.into_make_service_with_connect_info::<SocketAddr>()) => result?
    }

    Ok(())
}
