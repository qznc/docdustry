use crate::{config::Config, database::init_db};
use ignore::Walk;
use log::{debug, error, info};
use std::{collections::HashMap, fs::read_to_string};

pub(crate) fn cmd_gen_db(cfg: &Config) {
    let db = init_db(&cfg.db_path).unwrap();
    let mut docs: HashMap<String, Entry> = HashMap::new();
    for src in cfg.get_sources() {
        read_md_files(&mut docs, src.as_path());
    }
    db.execute("BEGIN TRANSACTION;").expect("begin");
    for did in docs.keys() {
        let entry = docs.get(did).unwrap();
        if !entry.raw_md.is_empty() {
            let tags = if entry.tags.is_empty() {
                String::new()
            } else {
                format!("§{}§", entry.tags.join("§"))
            };
            let query = "INSERT OR REPLACE INTO documents (did,raw,tags) VALUES (?,?,?);";
            let mut stmt = db.prepare(query).unwrap();
            stmt.bind((1, did.as_str())).unwrap();
            stmt.bind((2, entry.raw_md.as_str())).unwrap();
            stmt.bind((3, tags.as_str())).unwrap();
            stmt.next().unwrap();
        } else {
            debug!("skip DID {} because empty", did);
        }
    }
    db.execute("COMMIT;").expect("commit");
}

fn read_md_files(docs: &mut HashMap<String, Entry>, src_path_base: &std::path::Path) {
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
                        let meta = parse_meta(&markdown);
                        let entry = Entry {
                            raw_md: markdown,
                            tags: meta.tags,
                        };
                        docs.insert(meta.did, entry);
                    }
                    Err(e) => {
                        error!("Failed to read {:?}: {}", file.to_str(), e);
                    }
                }
            }
            Err(err) => error!("Not an entry: {}", err),
        }
    }
    info!(
        "Now {} docs after reading {}",
        docs.len(),
        src_path_base.display()
    );
}

struct Entry {
    raw_md: String,
    tags: Vec<String>,
}

struct Meta {
    did: String,
    tags: Vec<String>,
}

fn parse_meta(raw: &String) -> Meta {
    let mut ret = Meta {
        did: String::new(),
        tags: vec![],
    };
    for line in raw.split("\n") {
        if let Some((k, v)) = line.split_once(":") {
            match k {
                "id" => {
                    ret.did = v.trim().to_string();
                }
                "tag" => ret.tags.push(v.trim().to_string()),
                _ => (),
            }
        }
        if ret.did.is_empty() {
            if let Some(title) = line.strip_prefix("# ") {
                // Markdown title can be substitute DID
                ret.did = title.replace(" ", "_").to_ascii_lowercase();
            }
        }
    }
    ret
}
