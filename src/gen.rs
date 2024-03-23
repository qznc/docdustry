use ignore::Walk;
use log::{error, info};
use md5;
use pulldown_cmark::html::push_html;
use pulldown_cmark::Parser;
use std::fs::{create_dir_all, read_to_string, write};
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
}

/// Read contents of file, convert Markdown to HTML, and store as html file in output directory
fn gen_markdown(file: &Path, output_dir: &PathBuf) -> Result<(), std::io::Error> {
    info!("gen_markdown {:?}", &file);

    // generate HTML
    let markdown_content = read_to_string(file)?;
    let parser = Parser::new(&markdown_content);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    // generate file
    let hash = generate_short_hash(file.to_str().unwrap());
    let basename = file.file_stem().expect("stem").to_str().expect("str");
    let output_file_name = format!("{}_{}.html", basename, hash);
    let output_file_path = output_dir.join(output_file_name);
    create_dir_all(&output_dir)?;
    write(output_file_path, html_output)?;
    Ok(())
}

/// A short hash to avoid conflicts of same file names in different directories
fn generate_short_hash(input: &str) -> String {
    let hash = md5::compute(input);
    let hex_string = format!("{:x}", hash);
    let short_hash = &hex_string[..6];
    short_hash.to_string()
}
