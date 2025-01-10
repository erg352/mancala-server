use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use reqwest::Client;

use crate::mancala::Game;

pub struct Match {
    pub game: Game,
    pub bots: (Bot, Bot),
}

#[derive(Clone)]
#[allow(unused)]
pub struct Bot {
    name: Arc<str>,
    address: SocketAddr,
}

#[derive(Clone)]
pub struct AppState {
    pub bots: Arc<Mutex<Vec<Bot>>>,
    pub matches: Arc<Mutex<Vec<Mutex<Match>>>>,

    // For sending messages to clients
    pub client: Client,

    pub database: Arc<Mutex<rusqlite::Connection>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            bots: Default::default(),
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
