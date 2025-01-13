use std::path::PathBuf;

use clap::Parser;

/// Server used for match making mancala games
#[derive(Parser)]
pub struct Args {
    /// The port the server should bind to.
    #[arg(short, long)]
    pub port: u16,

    /// Path to the database used to store bot data and information.
    #[arg(short, long)]
    pub database: PathBuf,

    /// Specifies the level of tracing for the server. Possible values are: TRACE,
    /// DEBUG, INFO, WARN and ERROR; with TRACE implying DEBUG and so on and so forth.
    /// The provided value is case insensitive.
    #[arg(short, long, default_value_t = tracing::Level::WARN)]
    pub log: tracing::Level,
}
