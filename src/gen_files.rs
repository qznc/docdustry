use log::{info, warn};
use serde_json;
use std::fs::{self, create_dir_all, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::config::Config;
use crate::gen_html::{read_md_files, Doc};

pub(crate) fn cmd_gen(cfg: &Config) {
    let output = cfg.output.clone();
    if !output.exists() {
        create_dir_all(&output).unwrap();
    }
    info!("output: {}", &output.display());
    let mut docs: Vec<Doc> = vec![];
    for src in cfg.get_sources() {
        read_md_files(&mut docs, src.as_path());
    }
    let template: Vec<&str> = TMPL.split(&"XXX").collect();
    for d in &docs {
        let output_file_path = output.join(&d.html_path());
        write_html_doc(&output_file_path, &template, &"../", &d).expect("write html");
    }

    write_index_file(&output, &template, &docs, cfg).unwrap();
    write_static_files(&output).unwrap();
    write_globals_file(&output, &docs).unwrap();
}

fn write_globals_file(output_dir: &PathBuf, docs: &Vec<Doc>) -> Result<(), std::io::Error> {
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

fn write_index_file(
    output_dir: &PathBuf,
    template: &Vec<&str>,
    docs: &Vec<Doc>,
    cfg: &Config,
) -> Result<(), std::io::Error> {
    let output_file_path = output_dir.join(&"index.html");
    if let Some(did) = cfg.frontpage.clone() {
        match docs.iter().find(|d| d.did == did) {
            Some(d) => {
                info!("output {}", &output_file_path.display());
                write_html_doc(&output_file_path, template, &"", d).unwrap();
                return Ok(());
            }
            None => {
                warn!("Frontpage not found: {}", did);
            }
        }
    };
    let mut doc = Doc::new(PathBuf::from(output_dir), PathBuf::from("index.html"));
    doc.html = "<p>Please search!</p>".to_string();
    write_html_doc(&output_file_path, template, &"", &doc).unwrap();
    Ok(())
}

fn write_html_doc(
    output_file_path: &PathBuf,
    template: &Vec<&str>,
    path_prefix: &str,
    d: &Doc,
) -> Result<(), std::io::Error> {
    create_dir_all(&output_file_path.parent().unwrap())?;
    let title = &d.title;
    let content = &d.html;
    let json: &str = &serde_json::to_string(&d)?;
    let fh = File::create(&output_file_path)?;
    let mut st = BufWriter::new(fh);
    st.write(template[0].as_bytes())?;
    st.write(title.as_bytes())?;
    st.write(template[1].as_bytes())?;
    st.write(path_prefix.as_bytes())?;
    st.write(template[2].as_bytes())?;
    st.write(path_prefix.as_bytes())?;
    st.write(template[3].as_bytes())?;
    st.write(path_prefix.as_bytes())?;
    st.write(template[4].as_bytes())?;
    st.write(json.as_bytes())?;
    st.write(template[5].as_bytes())?;
    st.write(content.as_bytes())?;
    st.write(template[6].as_bytes())?;
    Ok(())
}

const TMPL: &str = r#"<!DOCTYPE html>
<html><head>
<title>XXX</title>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" /></head>
<link href="XXXdocdustry_static/default.css" rel="stylesheet">
<script src="XXXdocdustry_static/default.js" type="text/javascript" defer></script>
<script src="XXXdocdustry_static/globals.js" type="text/javascript" defer></script>
<script type="text/javascript">const DOCDUSTRY_LOCALS = XXX;</script>
<body><header></header>
<div class="page">
<section class="main">XXX</section>
</div>
<footer></footer></body></html>"#;
const CSS: &'static [u8] = include_bytes!("default.css");
const JS: &'static [u8] = include_bytes!("default.js");
