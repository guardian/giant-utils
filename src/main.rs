use std::path::PathBuf;

use clap::{Parser, Subcommand};
use giant_api::get_or_insert_collection;
use hash::hash_file;
use model::{
    cli_output::{CliResult, OutputFormat},
    exit_code::ExitCode, uri::Uri
};

mod auth_store;
mod giant_api;
mod hash;
mod model;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Set the output format
    #[clap(arg_enum, short, long, default_value_t=OutputFormat::TSV)]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// Hash a file at the path provided to produce a Giant ID
    Hash { path: String },
    /// Login to the Giant instance at the provided URI with an auth token
    Login { uri: String, token: String },
    /// Check if the provided hash is in Giant, and you have permission to see it
    CheckHash { uri: String, hash: String },
    /// Check if the provided file is in Giant, and you have permission to see it
    CheckFile { uri: String, path: String },
    Ingest {
        uri: String,
        ingestion_uri: String,
        path: PathBuf,
        languages: String,
        bucket: String,
        #[clap(default_value = "aws:kms")]
        sse_algorithm: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let format = &cli.format;

    match &cli.command {
        Commands::Hash { path } => {
            CliResult::new(hash_file(path.clone()), ExitCode::HashFailed).print_or_exit(format);
        }
        Commands::Login { uri, token } => {
            CliResult::new(auth_store::set(uri, token), ExitCode::SetAuthTokenFailed).exit();
        }
        Commands::CheckHash { uri, hash } => {
            CliResult::new(giant_api::check_hash_exists(uri, hash), ExitCode::ApiFailed)
                .print_or_exit(format);
        }
        Commands::CheckFile { uri, path } => {
            let file_exists = (|| {
                let hash = hash_file(path.clone())?;
                giant_api::check_hash_exists(uri, &hash.hash)
            })();

            CliResult::new(file_exists, ExitCode::ApiFailed).print_or_exit(format);
        }
        Commands::Ingest {
            uri,
            ingestion_uri,
            path: _,
            languages: _,
            bucket: _,
            sse_algorithm:_ ,
        } => {
            let ingestion_uri = Uri::parse(ingestion_uri).unwrap();
            let collection = get_or_insert_collection(uri, &ingestion_uri);
            println!("{:#?}", collection.unwrap());
        }
    }
}
