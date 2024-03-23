use ignore::Walk;
use log::{error, info};
use md5;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Parser, Tag};
use serde_json;
use std::fs::{self, create_dir_all, read_to_string, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize)]
struct Doc {
    title: String,
    out_path: PathBuf,
    links: Vec<String>,
    id: String,
}

pub(crate) fn cmd_gen(sources: PathBuf, output: PathBuf) {
    let mut docs: Vec<Doc> = vec![];
    parse_md_and_write(sources, &output, &mut docs);
    write_static_files(&output).unwrap();
    write_globals_file(&output, docs).unwrap();
}

fn parse_md_and_write(sources: PathBuf, output: &PathBuf, docs: &mut Vec<Doc>) {
    for result in Walk::new(sources) {
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
                info!("found {}", entry.path().display());
                match gen_markdown(p, output) {
                    Err(e) => error!("Markdown generation failed: {}", e),
                    Ok(doc) => docs.push(doc),
                }
            }
            Err(err) => error!("Not an entry: {}", err),
        }
    }
}

fn write_globals_file(output_dir: &PathBuf, docs: Vec<Doc>) -> Result<(), std::io::Error> {
    let mut c = String::new();
    c.push_str("const DOCDUSTRY_GLOBALS = {");

    // doc data
    c.push_str("docs:[\n");
    for doc in docs {
        c.push_str(&serde_json::to_string(&doc)?);
        c.push_str(",\n");
    }
    c.push_str("],");

    c.push_str("};");
    let path = output_dir.join("docdustry_static/globals.js");
    fs::write(path, c)?;
    Ok(())
}

fn write_static_files(output_dir: &PathBuf) -> Result<(), std::io::Error> {
    let dir = output_dir.join("docdustry_static");
    create_dir_all(&dir)?;
    fs::write(dir.join("default.css"), CSS)?;
    fs::write(dir.join("default.js"), JS)?;
    Ok(())
}

const TMPL: [&'static str; 4] = [
    r#"<!DOCTYPE html>
<html><head>
<title>"#,
    r#"</title>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" /></head>
<link href="../docdustry_static/default.css" rel="stylesheet">
<script src="../docdustry_static/default.js" type="text/javascript" defer></script>
<script src="../docdustry_static/globals.js" type="text/javascript" defer></script>
<script type="text/javascript">const DOCDUSTRY_LOCALS = "#,
    r#";</script>
<body><header></header>
<section class="main">"#,
    r#"</section>
<footer></footer></body></html>"#,
];
const CSS: &'static [u8] = include_bytes!("default.css");
const JS: &'static [u8] = include_bytes!("default.js");

/// Read contents of file, convert Markdown to HTML, and store as html file in output directory
fn gen_markdown(file: &Path, output_dir: &PathBuf) -> Result<Doc, std::io::Error> {
    // generate HTML
    let markdown_content = read_to_string(file)?;
    let mut meta_info = parse_meta_info(Parser::new(&markdown_content));
    let mut html_output = String::new();
    push_html(&mut html_output, Parser::new(&markdown_content));

    // generate file
    let out_path = create_output_file(file, output_dir)?;
    meta_info.id = if meta_info.id.is_empty() {
        let hash = md5::compute(&out_path.to_str().unwrap());
        format!("did:{:x}", hash)
    } else {
        meta_info.id
    };
    write_html(File::create(&out_path)?, html_output, &meta_info)?;
    Ok(Doc {
        title: meta_info.title,
        out_path: out_path.strip_prefix(output_dir).unwrap().to_path_buf(),
        links: meta_info.links,
        id: meta_info.id,
    })
}

fn write_html(fh: File, html: String, meta_info: &DocMetaInfo) -> Result<(), std::io::Error> {
    let mut st = BufWriter::new(fh);
    st.write(TMPL[0].as_bytes())?;
    st.write(meta_info.title.as_bytes())?;
    st.write(TMPL[1].as_bytes())?;
    st.write(serde_json::to_string(&meta_info)?.as_bytes())?;
    st.write(TMPL[2].as_bytes())?;
    st.write(html.as_bytes())?;
    st.write(TMPL[3].as_bytes())?;
    Ok(())
}

fn create_output_file(file: &Path, output_dir: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let dir_hash = generate_short_hash(file.parent().unwrap().to_str().unwrap());
    let dir = output_dir.join(dir_hash);
    let basename = file.file_stem().expect("stem").to_str().expect("str");
    let output_file_name = format!("{}.html", basename);
    let output_file_path = dir.join(output_file_name);
    info!("gen_markdown {:?} to {:?}", &file, &output_file_path);
    create_dir_all(&dir)?;
    Ok(output_file_path)
}

#[derive(serde::Serialize)]
struct DocMetaInfo {
    title: String,
    id: String,
    status: String,

    #[serde(skip)]
    links: Vec<String>,
    tags: Vec<String>,
}

impl DocMetaInfo {
    fn new() -> DocMetaInfo {
        DocMetaInfo {
            title: "<unknown>".to_string(),
            id: String::new(),
            status: String::new(),
            links: vec![],
            tags: vec![],
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
                        self.id = v.trim().to_string();
                    }
                    "tag" => {
                        self.tags.push(v.trim().to_string());
                    }
                    _ => (),
                }
            }
        }
    }
}

fn parse_meta_info(mut parser: Parser<'_>) -> DocMetaInfo {
    let mut ret = DocMetaInfo::new();
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
                        ret.title = t.to_string();
                    }
                }
                Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                    if lang != CowStr::from("docdustry-docmeta") {
                        continue;
                    }
                    if let Some(Event::Text(t)) = parser.next() {
                        ret.parse_meta(t.to_string());
                    }
                }
                Tag::Link {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    if dest_url.starts_with(&"https://") || dest_url.starts_with(&"http://") {
                        continue;
                    }
                    ret.links.push(dest_url.to_string());
                }
                _ => (), // don't care
            },
            _ => (), // don't care
        }
    }
    ret
}

/// A short hash to avoid conflicts of same file names in different directories
fn generate_short_hash(input: &str) -> String {
    let hash = md5::compute(input);
    let hex_string = format!("{:x}", hash);
    let short_hash = &hex_string[..6];
    short_hash.to_string()
}
