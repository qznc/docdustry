use std::path::PathBuf;

use sqlite::{Connection, Result};

use crate::{
    config::Config,
    gen_html::{read_md_files, Doc},
};

pub(crate) fn cmd_gen_db(cfg: &Config) {
    let db = init_db(&cfg.db_path).unwrap();
    let mut docs: Vec<Doc> = vec![];
    for src in cfg.get_sources() {
        read_md_files(&mut docs, src.as_path());
    }
    db.execute("BEGIN TRANSACTION;").expect("begin");
    for d in &docs {
        let query = "INSERT INTO documents (did,raw) VALUES (?,?);";
        let mut stmt = db.prepare(query).unwrap();
        stmt.bind((1, d.did.as_str())).unwrap();
        stmt.bind((2, d.raw.as_str())).unwrap();
        stmt.next().unwrap();
    }
    db.execute("COMMIT;").expect("commit");
}

/// Create and init sqlite3 database if necessary
fn init_db(db_path: &PathBuf) -> Result<Connection> {
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
