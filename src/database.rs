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
            raw TEXT NOT NULL,
            title TEXT NOT NULL,
            tags TEXT
        )",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS requirements (
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            status TEXT NOT NULL
        )",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS relations (
            src TEXT NOT NULL,
            verb TEXT NOT NULL,
            tgt TEXT NOT NULL,
            UNIQUE(src, verb, tgt)
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
            debug!("did {} not found in database", did);
            None
        }
    }

    pub fn by_tag(&self, tag: &str) -> Vec<String> {
        debug!("Get documents by tag {}", tag);
        let mut md = vec![];
        let tag_wrap = format!("%§{}§%", tag.trim());
        let query = "SELECT raw FROM documents WHERE tags LIKE ?1";
        let mut statement = self.con.prepare(query).unwrap();
        statement.bind((1, tag_wrap.as_str())).unwrap();
        while let Ok(State::Row) = statement.next() {
            let raw = statement.read("raw").unwrap();
            md.push(raw);
        }
        debug!("Got {} documents", md.len());
        md
    }

    pub fn backlinks(&self, did: &str) -> Vec<String> {
        let rels = self.relations_with_tgt(did);
        let mut sources = vec![];
        for rel in rels {
            if rel.verb != "links" {
                continue;
            }
            sources.push(rel.src);
        }
        sources
    }

    pub fn relations_with_src(&self, did: &str) -> Vec<RelationTarget> {
        let mut rels = vec![];
        let query = "SELECT verb,tgt FROM relations WHERE src == ?1";
        let mut statement = self.con.prepare(query).unwrap();
        statement.bind((1, did)).unwrap();
        while let Ok(State::Row) = statement.next() {
            let tgt = statement.read("tgt").unwrap();
            let verb = statement.read("verb").unwrap();
            rels.push(RelationTarget { tgt, verb });
        }
        rels
    }

    pub fn relations_with_tgt(&self, did: &str) -> Vec<RelationSource> {
        let mut rels = vec![];
        let query = "SELECT src,verb FROM relations WHERE tgt == ?1";
        let mut statement = self.con.prepare(query).unwrap();
        statement.bind((1, did)).unwrap();
        while let Ok(State::Row) = statement.next() {
            let src = statement.read("src").unwrap();
            let verb = statement.read("verb").unwrap();
            rels.push(RelationSource { verb, src });
        }
        rels
    }

    pub fn search(&self, search_term: &str) -> Vec<SearchResult> {
        let mut ret = vec![];
        let sql_search_term = format!("%{}%", search_term);
        let query = "SELECT did,title FROM documents WHERE raw LIKE ?1";
        let mut statement = self.con.prepare(query).unwrap();
        statement.bind((1, sql_search_term.as_str())).unwrap();
        while let Ok(State::Row) = statement.next() {
            let did: String = statement.read("did").unwrap();
            let title: String = statement.read("title").unwrap();
            if did.is_empty() {
                continue;
            }
            ret.push(SearchResult {
                did: did.clone(),
                title: title.clone(),
            });
        }
        debug!("Search found {} results", ret.len());
        ret
    }
}

pub struct SearchResult {
    pub did: String,
    pub title: String,
}

pub struct RelationTarget {
    pub verb: String,
    pub tgt: String,
}
pub struct RelationSource {
    pub src: String,
    pub verb: String,
}
