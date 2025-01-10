use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    #[arg(short, long, default_value_t = tracing::Level::WARN)]
    pub log: tracing::Level,
}
