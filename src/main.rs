use clap::Parser;
use log::error;
use std::path::{Path, PathBuf};

mod gen_files;
mod gen_html;
mod spam_md;

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

fn main() {
    env_logger::init();
    let args = Cli::parse();
    println!("Command: {}", args.command);

    match args.command.as_str() {
        "gen" => gen_files::cmd_gen(args.sources, args.output),
        "spam_md" => spam_md::generate_random_markdown_files(Path::new(&"spam"), 100, 100),
        _ => error!("Unknown command {}", args.command),
    }
}
