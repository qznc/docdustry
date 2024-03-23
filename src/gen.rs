use ignore::Walk;
use log::{error, info};
use md5;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Parser, Tag};
use std::fs::{self, create_dir_all, read_to_string, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

pub(crate) fn cmd_gen(sources: PathBuf, output: PathBuf) {
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
                if let Err(e) = gen_markdown(p, &output) {
                    error!("Markdown generation failed: {}", e);
                }
            }
            Err(err) => error!("ERROR: {}", err),
        }
    }
    write_static_files(&output).ok();
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
<script type="text/javascript">val DOCDUSTRY = {"#,
    r#"};</script>
<body><header></header>
<section class="main">"#,
    r#"</section>
<footer></footer></body></html>"#,
];
const CSS: &'static [u8] = include_bytes!("default.css");
const JS: &'static [u8] = include_bytes!("default.js");

/// Read contents of file, convert Markdown to HTML, and store as html file in output directory
fn gen_markdown(file: &Path, output_dir: &PathBuf) -> Result<(), std::io::Error> {
    // generate HTML
    let markdown_content = read_to_string(file)?;
    let meta_info = parse_meta_info(Parser::new(&markdown_content));
    let mut html_output = String::new();
    push_html(&mut html_output, Parser::new(&markdown_content));

    // generate file
    let fh = create_output_file(file, output_dir)?;
    write_html(fh, html_output, meta_info)
}

fn write_html(fh: File, html_output: String, meta_info: DocMetaInfo) -> Result<(), std::io::Error> {
    let mut st = BufWriter::new(fh);
    st.write(TMPL[0].as_bytes())?;
    st.write(meta_info.title.as_bytes())?;
    st.write(TMPL[1].as_bytes())?;
    // TODO: inject meta info for js use
    st.write(TMPL[2].as_bytes())?;
    st.write(html_output.as_bytes())?;
    st.write(TMPL[3].as_bytes())?;
    Ok(())
}

fn create_output_file(file: &Path, output_dir: &PathBuf) -> Result<File, std::io::Error> {
    let dir_hash = generate_short_hash(file.parent().unwrap().to_str().unwrap());
    let dir = output_dir.join(dir_hash);
    let basename = file.file_stem().expect("stem").to_str().expect("str");
    let output_file_name = format!("{}.html", basename);
    let output_file_path = dir.join(output_file_name);
    info!("gen_markdown {:?} to {:?}", &file, &output_file_path);
    create_dir_all(&dir)?;
    let fh = File::create(output_file_path)?;
    Ok(fh)
}

struct DocMetaInfo {
    title: String,
    meta: String,
}

impl DocMetaInfo {
    pub fn is_complete(&self) -> bool {
        !self.title.is_empty() && !self.meta.is_empty()
    }
}

fn parse_meta_info(mut parser: Parser<'_>) -> DocMetaInfo {
    let mut ret = DocMetaInfo {
        title: String::new(),
        meta: String::new(),
    };
    while let Some(event) = parser.next() {
        if ret.is_complete() {
            break;
        }
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
                        ret.meta = t.to_string();
                    }
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
