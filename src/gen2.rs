use ignore::Walk;
use log::{error, info};
use pulldown_cmark::{html::push_html, Parser};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag};
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct Doc {
    pub title: String,
    pub status: String,
    pub links: Vec<String>,
    pub tags: Vec<String>,

    /// document ID, the unique identifier for linking to it
    pub did: String,
    #[serde(skip)]
    html: String,
    #[serde(skip)]
    pub src_path_rel: PathBuf,
    #[serde(skip)]
    src_path_base: PathBuf,
}

impl Doc {
    fn new(src_path_base: &PathBuf, src_path_rel: PathBuf) -> Doc {
        Doc {
            src_path_rel,
            src_path_base: src_path_base.to_path_buf(),
            html: String::with_capacity(1000),
            title: "<unknown>".to_string(),
            links: vec![],
            tags: vec![],
            did: String::new(),
            status: String::new(),
        }
    }

    fn gen_html(&mut self) -> Result<(), io::Error> {
        let file = self.src_path_base.join(&self.src_path_rel);
        let markdown_content = read_to_string(file)?;
        self.parse_meta_data(Parser::new(&markdown_content));
        // TODO: for inlining other pages we to track dependencies
        push_html(&mut self.html, Parser::new(&markdown_content));
        Ok(())
    }

    fn parse_meta_data(&mut self, mut parser: Parser) {
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading {
                        level,
                        id: _,
                        classes: _,
                        attrs: _,
                    } => {
                        if level != HeadingLevel::H1 {
                            continue;
                        }
                        if let Some(Event::Text(t)) = parser.next() {
                            self.title = t.to_string();
                        }
                    }
                    Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                        if lang != CowStr::from("docdustry-docmeta") {
                            continue;
                        }
                        if let Some(Event::Text(t)) = parser.next() {
                            self.parse_meta(t.to_string());
                        }
                    }
                    Tag::Link {
                        link_type: _,
                        dest_url,
                        title: _,
                        id: _,
                    } => {
                        if dest_url.starts_with(&"https://")
                            || dest_url.starts_with(&"http://")
                            || dest_url.starts_with(&"#")
                        {
                            continue;
                        }
                        self.links.push(dest_url.to_string());
                    }
                    _ => (), // don't care
                },
                _ => (), // don't care
            }
        }
        // post-processing
        if self.did.is_empty() {
            let mut ctx = md5::Context::new();
            ctx.consume(&self.src_path_rel.as_os_str().as_encoded_bytes());
            ctx.consume(&self.title);
            let hash = ctx.compute();
            self.did = format!("{:x}", hash);
        }
    }

    fn parse_meta(&mut self, meta: String) {
        for line in meta.split("\n") {
            if let Some((k, v)) = line.split_once(":") {
                match k {
                    "status" => {
                        self.status = v.trim().to_string();
                    }
                    "id" => {
                        self.did = v.trim().to_string();
                    }
                    "tag" => {
                        self.tags.push(v.trim().to_string());
                    }
                    _ => (),
                }
            }
        }
    }

    pub(crate) fn write_html(&self, st: &mut BufWriter<File>) -> Result<usize, io::Error> {
        st.write(self.html.as_bytes())
    }
}

fn _out_path_from_src(src_path_rel: &PathBuf) -> PathBuf {
    let mut out_path_rel = PathBuf::new();
    let hash = {
        let hash = md5::compute(src_path_rel.as_os_str().as_encoded_bytes());
        let hex_string = format!("{:x}", hash);
        let short_hash = &hex_string[..6];
        short_hash.to_string()
    };
    out_path_rel.push(hash);
    let basename = src_path_rel
        .file_stem()
        .expect("stem")
        .to_str()
        .expect("str");
    let out_file_name = format!("{}.html", basename);
    out_path_rel.push(out_file_name);
    out_path_rel
}

struct HtmlConverter {
    docs: Vec<Doc>,
}

impl HtmlConverter {
    pub fn new() -> HtmlConverter {
        HtmlConverter { docs: vec![] }
    }

    pub fn read_md_files(&mut self, src_path_base: PathBuf) -> Result<(), io::Error> {
        for result in Walk::new(&src_path_base) {
            match result {
                Ok(entry) => {
                    let t = entry.file_type().expect("file type");
                    if t.is_dir() {
                        continue;
                    }
                    let p = entry.path();
                    if p.extension().unwrap() != "md" {
                        continue;
                    }
                    let src_path_rel = entry
                        .path()
                        .strip_prefix(&src_path_base)
                        .expect("is prefix")
                        .to_path_buf();
                    self.docs.push(Doc::new(&src_path_base, src_path_rel));
                }
                Err(err) => error!("Not an entry: {}", err),
            }
        }
        info!("Found {} md files", self.docs.len());
        for d in &mut self.docs {
            d.gen_html()?;
        }
        Ok(())
    }
}

pub fn read_md_files(src_path_base: PathBuf) -> Result<Vec<Doc>, io::Error> {
    let mut conv = HtmlConverter::new();
    conv.read_md_files(src_path_base)?;
    Ok(conv.docs)
}
