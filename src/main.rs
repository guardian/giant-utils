use clap::{Parser, Subcommand};
use hash::hash_file;

mod hash;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Hash { path: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hash { path } => match hash_file(path) {
            Ok(digest) => println!("{digest}"),
            Err(e) => println!("Failed to hash file: {}", e),
        },
    }
}
