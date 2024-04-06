use clap::Parser;
use ini::Ini;
use log::{error, info, warn};
use std::path::{Path, PathBuf};

mod config;
mod gen_files;
mod gen_html;
mod spam_md;

#[derive(Parser)]
struct Cli {
    /// Subcommand
    #[arg(default_value = "gen")]
    command: String,

    /// Config file in ini format
    #[arg(short = 'i', long = "ini")]
    ini: Option<PathBuf>,
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
                            } else {
                                println!("[{:?}] {}:{}", sec, k, v);
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

    match args.command.as_str() {
        "gen" => gen_files::cmd_gen(&cfg),
        "spam_md" => spam_md::generate_random_markdown_files(Path::new(&"spam"), 100, 100),
        _ => error!("Unknown command {}", args.command),
    }
}
