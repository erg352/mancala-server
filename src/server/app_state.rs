use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    // For sending messages to clients
    pub client: Client,

    pub database: Arc<Mutex<rusqlite::Connection>>,
}

impl AppState {
    pub fn new(database_path: &Path) -> Self {
        Self {
            client: Default::default(),
            database: Arc::new(Mutex::new(open_database(database_path))),
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
