use std::sync::{Arc, Mutex};

use reqwest::Client;

use crate::mancala::Game;

pub struct Match {
    pub game: Game,
    pub ports: (u16, u16),
}

#[derive(Clone)]
pub struct AppState {
    pub players: Arc<Mutex<Vec<u16>>>,
    pub matches: Arc<Mutex<Vec<Mutex<Match>>>>,

    // For sending messages to clients
    pub client: Client,

    pub database: Arc<Mutex<rusqlite::Connection>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            players: Default::default(),
            matches: Default::default(),
            client: Default::default(),
            database: Arc::new(Mutex::new(open_database())),
        }
    }
}

fn open_database() -> rusqlite::Connection {
    let database = rusqlite::Connection::open("data.db").unwrap();

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
