use log::info;
use serde_json;
use std::fs::{self, create_dir_all, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

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
    for d in &docs {
        write_html_doc(&output, &d).expect("write html");
    }

    write_static_files(&output).unwrap();
    write_globals_file(&output, &docs).unwrap();
}

fn write_html_doc(output_dir: &PathBuf, d: &Doc) -> Result<(), std::io::Error> {
    let html_path = d.html_path();
    let out_path = create_output_file(&html_path, output_dir)?;
    let fh = File::create(&out_path)?;
    let mut st = BufWriter::new(fh);
    st.write(TMPL[0].as_bytes())?;
    st.write(d.title.as_bytes())?;
    st.write(TMPL[1].as_bytes())?;
    st.write(serde_json::to_string(&d)?.as_bytes())?;
    st.write(TMPL[2].as_bytes())?;
    d.write_html(&mut st)?;
    st.write(TMPL[3].as_bytes())?;
    Ok(())
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
    fs::write(output_dir.join("index.html"), INDEX_HTML)?;
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
<div class="page">
<section class="main">"#,
    r#"</section>
</div>
<footer></footer></body></html>"#,
];
const CSS: &'static [u8] = include_bytes!("default.css");
const JS: &'static [u8] = include_bytes!("default.js");
const INDEX_HTML: &'static [u8] = include_bytes!("index.html");

fn create_output_file(html_path: &Path, output_dir: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let output_file_path = output_dir.join(html_path);
    create_dir_all(&output_file_path.parent().unwrap())?;
    Ok(output_file_path)
}
