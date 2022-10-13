use std::collections::HashSet;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hash::hash_file;
use ingestion::{
    ingestion_upload::ingestion_upload,
    progress_reader::{empty_progress_reader, progress_reader_from_path},
};
use itertools::Itertools;
use model::{
    cli_error::CliError,
    cli_output::{CliResult, OutputFormat},
    exit_code::FailureExitCode,
    lang::Language,
    uri::Uri,
};
use services::giant_api;
use tokio::runtime::Runtime;

mod auth_store;
mod hash;
mod ingestion;
mod model;
mod services;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Set the output format
    #[clap(arg_enum, short, long, default_value_t=OutputFormat::Tsv)]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// Hash a file at the path provided to produce a Giant ID
    Hash {
        /// The file you wish to hash
        path: String,
    },
    /// Login to the Giant instance at the provided URI with an auth token
    Login {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: String,
        /// Your auth token, found on the about page
        token: String,
    },
    /// Check if the provided hash is in Giant, and you have permission to see it
    CheckHash {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: String,
        /// The resource hash you wish to check exists in Giant
        hash: String,
    },
    /// Check if the provided file is in Giant, and you have permission to see it
    CheckFile {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: String,
        /// The path to the file on your local disk
        path: String,
    },
    /// Upload all files in a directory to Giant
    Ingest {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: String,
        /// The ingestion URI for your upload, in the form "collection/ingestion"
        ingestion_uri: String,
        /// The base path for your upload
        path: PathBuf,
        /// A comma sepearted list of the languages in the files
        languages: String,
        /// The bucket you wish to upload to
        bucket: String,
        /// Continue from a previous ingestion using its log
        #[clap(short, long)]
        progress_from: Option<PathBuf>,
    },
    /// List the blobs in a collection
    DeleteCollection {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: String,
        /// The collection you want to delete
        collection: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let format = &cli.format;

    match &cli.command {
        Commands::Hash { path } => {
            CliResult::new(hash_file(path.clone()), FailureExitCode::Hash).print_or_exit(format);
        }
        Commands::Login { giant_uri, token } => {
            CliResult::new(
                auth_store::set(giant_uri, token),
                FailureExitCode::SetAuthToken,
            )
            .exit();
        }
        Commands::CheckHash { giant_uri, hash } => {
            CliResult::new(
                giant_api::check_hash_exists(giant_uri, hash),
                FailureExitCode::Api,
            )
            .print_or_exit(format);
        }
        Commands::CheckFile { giant_uri, path } => {
            let file_exists = (|| {
                let hash = hash_file(path.clone())?;
                giant_api::check_hash_exists(giant_uri, &hash.hash)
            })();

            CliResult::new(file_exists, FailureExitCode::Api).print_or_exit(format);
        }
        Commands::Ingest {
            giant_uri,
            ingestion_uri,
            path,
            languages,
            bucket,
            progress_from,
        } => {
            // I'm sure we can do better than this.
            let languages: Vec<Language> = languages
                .split(',')
                .map(|l| match l {
                    "arabic" => Language::Arabic,
                    "english" => Language::English,
                    "french" => Language::French,
                    "german" => Language::German,
                    "portuguese" => Language::Portuguese,
                    "russian" => Language::Russian,
                    _ => panic!("Invalid language!"),
                })
                .collect();

            let result: Result<(), CliError> = (|| {
                let progress_reader = match progress_from {
                    Some(path) => progress_reader_from_path(path)?,
                    None => empty_progress_reader(),
                };

                let ingestion_uri = Uri::parse(ingestion_uri)?;
                let collection = giant_api::get_or_insert_collection(giant_uri, &ingestion_uri)?;

                println!("Checking ingestion");
                giant_api::get_or_insert_ingestion(
                    giant_uri,
                    &ingestion_uri,
                    &collection,
                    path.to_path_buf(),
                    languages.to_vec(),
                )?;

                println!("Starting crawl");
                let rt = Runtime::new()?;
                rt.block_on(async {
                    // Walk file tree and upload files
                    ingestion_upload(
                        ingestion_uri,
                        &languages,
                        path,
                        bucket,
                        progress_reader,
                        format,
                    )
                    .await
                })
            })();

            CliResult::new(result, FailureExitCode::Upload).print_or_exit(format);
        }
        Commands::DeleteCollection {
            giant_uri,
            collection,
        } => {
            let result: Result<(), CliError> = (|| {
                // Returns a maximum of 500 results,
                // so we need to loop until we've deleted them all.
                let mut blobs = giant_api::get_blobs_in_collection(giant_uri, collection)?;

                while !blobs.is_empty() {
                    for blob in blobs {
                        println!("Blob is in collections: {:?}", blob.collections);

                        // TODO: would HashSet be more efficient here?
                        let other_collections: Vec<String> = blob.collections
                            .into_iter()
                            .filter(|c| c != collection)
                            .collect();

                        if !other_collections.is_empty() {
                            println!(
                                "Blob {} exists in other collections, will also delete from: {:?}",
                                blob.uri, other_collections
                            );
                        }
                        println!("Deleting blob {}", blob.uri);
                        giant_api::delete_blob(giant_uri, &blob.uri)?;
                        println!("Deleted blob {}", blob.uri);
                    }
                    blobs = giant_api::get_blobs_in_collection(giant_uri, collection)?;
                }

                println!("Deleting collection {}", collection);
                giant_api::delete_collection(giant_uri, collection)?;
                println!("Deleted collection {}", collection);

                return Ok(());
            })();

            CliResult::new(result, FailureExitCode::Api).print_or_exit(format);
        }
    }
}
