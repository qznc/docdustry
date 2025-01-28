use clap::{Parser, Subcommand};
use ini::Ini;
use log::{info, warn};
use std::path::{Path, PathBuf};

mod config;
mod gen_db;
mod gen_files;
mod gen_html;
mod spam_md;

#[derive(Parser)]
struct Cli {
    /// Subcommand
    #[clap(subcommand)]
    command: Command,

    /// Config file in ini format
    #[arg(short = 'i', long = "ini")]
    ini: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate HTML documentation
    Gen {},
    /// Generate Sqlite3 table
    GenDB {},
    /// Generate random spam
    SpamMd {},
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    let mut cfg = config::Config::new();

    if let Some(ini_path) = args.ini {
        if ini_path.exists() {
            let i = Ini::load_from_file(ini_path).unwrap();
            for (sec, prop) in i.iter() {
                match sec {
                    Some("gen") => {
                        for (k, v) in prop.iter() {
                            if k == "sources" {
                                cfg.push_source_dir(PathBuf::from(v));
                            } else if k == "output" {
                                cfg.output = PathBuf::from(v);
                            } else if k == "frontpage" {
                                cfg.frontpage = Some(v.to_string());
                            } else if k == "theme" {
                                cfg.theme = Some(PathBuf::from(v.to_string()));
                            } else {
                                warn!("Unknown config [gen] {}:{}", k, v);
                            }
                        }
                    }
                    Some(_) => (),
                    None => (),
                }
            }
        } else {
            warn!("Config file does not exist: {}", ini_path.display());
        }
    } else {
        info!("No config file given.");
    }

    match args.command {
        Command::Gen {} => gen_files::cmd_gen(&cfg),
        Command::GenDB {} => gen_db::cmd_gen_db(&cfg),
        Command::SpamMd {} => spam_md::generate_random_markdown_files(Path::new(&"spam"), 100, 100),
    }
}
