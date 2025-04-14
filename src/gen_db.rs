use crate::{config::Config, database::init_db};
use ignore::Walk;
use log::{debug, error, info};
use std::{collections::HashMap, fs::read_to_string};

pub(crate) fn cmd_gen_db(cfg: &Config) {
    let db = init_db(&cfg.db_path).unwrap();
    let mut docs: HashMap<String, String> = HashMap::new();
    for src in cfg.get_sources() {
        read_md_files(&mut docs, src.as_path());
    }
    db.execute("BEGIN TRANSACTION;").expect("begin");
    for did in docs.keys() {
        let raw = docs.get(did).unwrap();
        if raw != "" {
            debug!("insert DID {}", did);
            let query = "INSERT OR REPLACE INTO documents (did,raw) VALUES (?,?);";
            let mut stmt = db.prepare(query).unwrap();
            stmt.bind((1, did.as_str())).unwrap();
            stmt.bind((2, raw.as_str())).unwrap();
            stmt.next().unwrap();
        } else {
            debug!("skip DID {} because empty", did);
        }
    }
    db.execute("COMMIT;").expect("commit");
}

fn read_md_files(docs: &mut HashMap<String, String>, src_path_base: &std::path::Path) {
    for result in Walk::new(&src_path_base) {
        match result {
            Ok(entry) => {
                let t = entry.file_type().expect("file type");
                if t.is_dir() {
                    continue;
                }
                let p = entry.path();
                match p.extension() {
                    Some(ext) => {
                        if ext != "md" {
                            continue;
                        }
                    }
                    None => continue,
                };
                let src_path_rel = entry
                    .path()
                    .strip_prefix(&src_path_base)
                    .expect("is prefix")
                    .to_path_buf();
                let file = src_path_base.join(&src_path_rel);
                match read_to_string(&file) {
                    Ok(markdown) => {
                        let did = parse_did(&markdown);
                        docs.insert(did, markdown);
                    }
                    Err(e) => {
                        error!("Failed to read {:?}: {}", file.to_str(), e);
                    }
                }
            }
            Err(err) => error!("Not an entry: {}", err),
        }
    }
    info!("Found {} md files", docs.len());
}

fn parse_did(meta: &String) -> String {
    for line in meta.split("\n") {
        if let Some((k, v)) = line.split_once(":") {
            match k {
                "id" => {
                    return v.trim().to_string();
                }
                _ => (),
            }
        }
    }
    "unknown".to_string()
}
