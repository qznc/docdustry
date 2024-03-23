use ignore::Walk;
use log::{error, info};
use md5;
use pulldown_cmark::html::push_html;
use pulldown_cmark::Parser;
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

const TMPL_BEFORE: &'static str = r#"<!DOCTYPE html>
<html><head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" /></head>
<link href="../docdustry_static/default.css" rel="stylesheet">
<script src="../docdustry_static/default.js" type="text/javascript" defer></script>
<body><header></header>
<section class="main">"#;
const TMPL_AFTER: &'static str = r#"</section>
<footer></footer></body></html>"#;
const CSS: &'static [u8] = include_bytes!("default.css");
const JS: &'static [u8] = include_bytes!("default.js");

/// Read contents of file, convert Markdown to HTML, and store as html file in output directory
fn gen_markdown(file: &Path, output_dir: &PathBuf) -> Result<(), std::io::Error> {
    // generate HTML
    let markdown_content = read_to_string(file)?;
    let parser = Parser::new(&markdown_content);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    // generate file
    let dir_hash = generate_short_hash(file.parent().unwrap().to_str().unwrap());
    let dir = output_dir.join(dir_hash);
    let basename = file.file_stem().expect("stem").to_str().expect("str");
    let output_file_name = format!("{}.html", basename);
    let output_file_path = dir.join(output_file_name);
    info!("gen_markdown {:?} to {:?}", &file, &output_file_path);
    create_dir_all(&dir)?;

    let fh = File::create(output_file_path)?;
    let mut st = BufWriter::new(fh);
    st.write(TMPL_BEFORE.as_bytes())?;
    st.write(html_output.as_bytes())?;
    st.write(TMPL_AFTER.as_bytes())?;
    Ok(())
}

/// A short hash to avoid conflicts of same file names in different directories
fn generate_short_hash(input: &str) -> String {
    let hash = md5::compute(input);
    let hex_string = format!("{:x}", hash);
    let short_hash = &hex_string[..6];
    short_hash.to_string()
}
