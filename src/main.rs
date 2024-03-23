use clap::Parser;
use log::error;
use std::path::PathBuf;

mod gen;

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
        "gen" => gen::cmd_gen(args.sources, args.output),
        _ => error!("Unknown command {}", args.command),
    }
}
