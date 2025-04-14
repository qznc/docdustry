use std::path::PathBuf;

use log::debug;
use sqlite::{Connection, Result, State};

use crate::config::Config;

/// Create and init sqlite3 database if necessary
pub fn init_db(db_path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS definitions (
            id INTEGER PRIMARY KEY,
            term TEXT NOT NULL UNIQUE,
            definition TEXT NOT NULL
        )",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            did TEXT NOT NULL UNIQUE,
            raw TEXT NOT NULL
        )",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS requirements (
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            status TEXT NOT NULL
        )",
    )?;

    Ok(conn)
}

pub struct Database {
    con: Connection,
}

impl Database {
    pub fn open(cfg: &Config) -> Result<Database> {
        let db = init_db(&cfg.db_path).unwrap();
        Ok(Database { con: db })
    }

    pub fn get_did(&self, did: &str) -> Option<String> {
        debug!("Get did {} from documents table", did);

        // Prepare the SQL query to select the `raw` text from the `documents` table
        let query = "SELECT raw FROM documents WHERE did = ?1";
        let mut statement = self.con.prepare(query).unwrap();
        statement.bind((1, did)).unwrap();

        if let Ok(State::Row) = statement.next() {
            Some(statement.read("raw").unwrap())
        } else {
            None
        }
    }
}
