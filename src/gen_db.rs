use crate::{config::Config, database::init_db};
use ignore::Walk;
use log::{debug, error, info};
use pulldown_cmark::{Event, HeadingLevel, Parser};
use std::{collections::HashMap, fs::read_to_string};

pub fn cmd_gen_db(cfg: &Config) {
    let db = init_db(&cfg.db_path).unwrap();
    let mut docs: HashMap<String, Entry> = HashMap::new();
    for src in cfg.get_sources() {
        read_md_files(&mut docs, src.as_path());
    }
    db.execute("BEGIN TRANSACTION;").expect("begin");
    for did in docs.keys() {
        let entry = docs.get(did).unwrap();
        insert_document(entry, &db, did);
    }
    db.execute("COMMIT;").expect("commit");
}

fn insert_document(entry: &Entry, db: &sqlite::Connection, did: &String) {
    if !entry.raw_md.is_empty() {
        let tags = if entry.tags.is_empty() {
            String::new()
        } else {
            format!("§{}§", entry.tags.join("§"))
        };
        let query = "INSERT OR REPLACE INTO documents (did,raw,tags,title) VALUES (?,?,?,?);";
        let mut stmt = db.prepare(query).unwrap();
        stmt.bind((1, did.as_str())).unwrap();
        stmt.bind((2, entry.raw_md.as_str())).unwrap();
        stmt.bind((3, tags.as_str())).unwrap();
        stmt.bind((4, entry.title.as_str())).unwrap();
        stmt.next().unwrap();

        for rel in &entry.relations {
            let query = "INSERT OR REPLACE INTO relations (src,verb,tgt) VALUES (?,?,?);";
            let mut stmt = db.prepare(query).unwrap();
            stmt.bind((1, rel.from.as_str())).unwrap();
            stmt.bind((2, rel.verb.as_str())).unwrap();
            stmt.bind((3, rel.to.as_str())).unwrap();
            stmt.next().unwrap();
        }
    } else {
        debug!("skip DID {} because empty", did);
    }
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
                        let meta = parse_markdown_to_meta(&markdown);
                        let entry = Entry {
                            raw_md: markdown,
                            tags: meta.tags,
                            relations: meta.relations,
                            title: meta.title,
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
    relations: Vec<Relation>,
    title: String,
}

pub struct Meta {
    pub did: String,
    pub tags: Vec<String>,
    pub relations: Vec<Relation>,
    pub title: String,
}

pub struct Relation {
    pub from: String,
    pub to: String,
    pub verb: String,
}

enum NextTextAction {
    Nothing,
    Title,
    MetaBlock,
}

pub fn parse_markdown_to_meta(raw: &String) -> Meta {
    let mut ret = Meta {
        did: String::new(),
        tags: vec![],
        relations: vec![],
        title: String::new(),
    };
    let mut link_targets = vec![];
    let mut parser = Parser::new(raw);
    let mut next_text_action = NextTextAction::Nothing;
    while let Some(event) = parser.next() {
        match event {
            Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Heading { level, .. } => {
                    if level == HeadingLevel::H1 && ret.title.is_empty() {
                        next_text_action = NextTextAction::Title;
                    }
                }
                pulldown_cmark::Tag::CodeBlock(kind) => match kind {
                    pulldown_cmark::CodeBlockKind::Indented => (),
                    pulldown_cmark::CodeBlockKind::Fenced(typ) => {
                        if typ.as_ref() == "docdustry-docmeta" {
                            next_text_action = NextTextAction::MetaBlock;
                        }
                    }
                },
                //pulldown_cmark::Tag::MetadataBlock(_) => todo!(),
                pulldown_cmark::Tag::Link { dest_url, .. } => {
                    if let Some(did) = dest_url.strip_prefix("did:") {
                        link_targets.push(did.to_string());
                    }
                }
                pulldown_cmark::Tag::Image { dest_url, .. } => {
                    if let Some(did) = dest_url.strip_prefix("did:") {
                        link_targets.push(did.to_string());
                    }
                }
                _ => (),
            },
            Event::Text(t) => match next_text_action {
                NextTextAction::Nothing => (),
                NextTextAction::Title => {
                    ret.title = t.to_string();
                    if ret.did.is_empty() {
                        // Markdown title can be substitute DID
                        ret.did = t.replace(" ", "_").to_ascii_lowercase();
                    }
                    next_text_action = NextTextAction::Nothing;
                }
                NextTextAction::MetaBlock => {
                    parse_meta_block(&t.to_string(), &mut ret);
                    next_text_action = NextTextAction::Nothing;
                }
            },
            _ => (),
        }
    }
    for did in link_targets {
        ret.relations.push(Relation {
            from: ret.did.clone(),
            to: did,
            verb: String::from("links"),
        })
    }
    ret
}

fn parse_meta_block(raw: &String, ret: &mut Meta) {
    for line in raw.split("\n") {
        if let Some((k, v)) = line.split_once(":") {
            match k {
                "id" => {
                    ret.did = v.trim().to_string();
                }
                "tag" => ret.tags.push(v.trim().to_string()),
                key => {
                    let relations = ["needs", "derived_from"];
                    if relations.contains(&key) {
                        ret.relations.push(Relation {
                            from: ret.did.clone(),
                            to: v.trim().to_string(),
                            verb: String::from(key),
                        });
                    };
                }
            }
        }
    }
}
