use clap::Parser;
use ignore::Walk;
use log::{error, info};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Subcommand
    #[arg(default_value = "gen")]
    command: String,

    /// Folder to search for markdown documents
    #[arg(short = 's', default_value = ".")]
    sources: PathBuf,

    /// Output folder to write generated documentation to
    #[arg(short = 'o', default_value = "out/")]
    output: PathBuf,
}

fn cmd_gen(args: Cli) {
    // If output dir doesn't exist, create it
    if !args.output.exists() {
        if let Err(err) = fs::create_dir_all(&args.output) {
            error!("Failed to create dir: {}", err);
        }
    }
    for result in Walk::new(args.sources) {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.
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
            }
            Err(err) => error!("ERROR: {}", err),
        }
    }
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    println!("Command: {}", args.command);

    match args.command.as_str() {
        "gen" => cmd_gen(args),
        _ => error!("Unknown command {}", args.command),
    }
}
