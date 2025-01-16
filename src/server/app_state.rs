use std::{net::SocketAddr, path::Path, sync::Arc};

use tokio::sync::Mutex;

use reqwest::Client;

use crate::mancala::Game;

#[derive(Clone, Debug)]
pub struct Bot {
    pub name: Arc<str>,
    pub id: u16,
    pub elo: u16,
    pub address: SocketAddr,
}

#[derive(Clone)]
pub struct Match {
    pub game: Game,
    pub players: [Bot; 2],
}

#[derive(Clone)]
pub struct AppState {
    // For sending messages to clients
    pub client: Client,

    pub database: Arc<Mutex<rusqlite::Connection>>,

    pub pending_bots: Arc<Mutex<Vec<Bot>>>,
}

impl AppState {
    pub fn new(database_path: &Path) -> Self {
        Self {
            client: Default::default(),
            database: Arc::new(Mutex::new(open_database(database_path))),
            pending_bots: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn open_database(path: &Path) -> rusqlite::Connection {
    let database = rusqlite::Connection::open(path).unwrap();

    let query = "
            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                elo INTEGER
            )
        ";
    database.execute(query, []).unwrap();

    database
}
