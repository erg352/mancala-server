use std::{collections::HashSet, hash::Hash, net::SocketAddr, path::Path, sync::Arc};

use tokio::sync::Mutex;

use reqwest::Client;
use tracing::error;

use crate::mancala::Game;

#[derive(Clone, Debug)]
pub struct Bot {
    pub name: Arc<str>,
    pub id: u16,
    pub elo: u16,
    pub address: SocketAddr,
}

impl PartialEq for Bot {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Bot {}

impl Hash for Bot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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
    pub connected_bots: Arc<Mutex<HashSet<Bot>>>,
}

impl AppState {
    pub fn new(database_path: &Path) -> Self {
        let database = match open_database(database_path) {
            Ok(database) => database,
            Err(error) => {
                error!("Could not load in database due to following error: \"{error}\", shutting down server.");
                std::process::exit(1);
            }
        };
        Self {
            client: Default::default(),
            database: Arc::new(Mutex::new(database)),
            pending_bots: Arc::new(Mutex::new(Vec::new())),
            connected_bots: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

fn open_database(path: &Path) -> rusqlite::Result<rusqlite::Connection> {
    let database = rusqlite::Connection::open(path)?;

    let query = "
            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                elo INTEGER
            )
        ";
    database.execute(query, [])?;

    Ok(database)
}
