use std::path::PathBuf;
use std::time::SystemTime;

use crate::giant_api::{GiantApiClient, ListBlobsFilter};
use clap::{Parser, Subcommand};
use hash::hash_file;
use humantime::format_duration;
use ingestion::{
    ingestion_upload::ingestion_upload,
    progress_reader::{empty_progress_reader, progress_reader_from_path},
};
use model::{
    cli_error::CliError,
    cli_output::{CliResult, OutputFormat},
    exit_code::FailureExitCode,
    lang::Language,
    uri::Uri,
};
use reqwest::Url;
use services::giant_api;

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
        giant_uri: Url,
        /// Your auth token, found on the about page
        token: String,
    },
    /// Check if the provided hash is in Giant, and you have permission to see it
    CheckHash {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: Url,
        /// The resource hash you wish to check exists in Giant
        hash: String,
    },
    /// Check if the provided file is in Giant, and you have permission to see it
    CheckFile {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: Url,
        /// The path to the file on your local disk
        path: String,
    },
    /// Upload all files in a directory to Giant
    Ingest {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: Url,
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
    /// List the blobs in a collection.
    /// **Currently only lists up to 500 blobs**
    ListBlobs {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: Url,
        /// The collection whose blobs you want to list
        collection: String,
        /// List all blobs, or filter to only those that also exist in collections other
        /// than the one you are listing.
        #[clap(arg_enum, short, long, default_value_t=ListBlobsFilter::All)]
        filter: ListBlobsFilter,
    },
    /// Delete a collection and all its contents
    DeleteCollection {
        /// The URI of your Giant server, e.g. https://playground.pfi.gutools.co.uk
        giant_uri: Url,
        /// The collection you want to delete
        collection: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let format = &cli.format;

    match &cli.command {
        Commands::Hash { path } => {
            CliResult::new(hash_file(path.clone()), FailureExitCode::Hash).print_or_exit(format);
        }
        Commands::Login { giant_uri, token } => {
            CliResult::new(
                auth_store::set(giant_uri.as_str(), token),
                FailureExitCode::SetAuthToken,
            )
            .exit();
        }
        Commands::CheckHash { giant_uri, hash } => {
            let mut client = GiantApiClient::new(giant_uri.clone());
            CliResult::new(client.check_hash_exists(hash).await, FailureExitCode::Api)
                .print_or_exit(format);
        }
        Commands::CheckFile { giant_uri, path } => {
            let file_exists = (|| async {
                let mut client = GiantApiClient::new(giant_uri.clone());
                let hash = hash_file(path.clone())?;
                client.check_hash_exists(&hash.hash).await
            })()
            .await;

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

            let result: Result<(), CliError> = (|| async {
                let mut client = GiantApiClient::new(giant_uri.clone());
                let progress_reader = match progress_from {
                    Some(path) => progress_reader_from_path(path)?,
                    None => empty_progress_reader(),
                };

                let ingestion_uri = Uri::parse(ingestion_uri)?;
                let collection = client.get_or_insert_collection(&ingestion_uri).await?;

                println!("Checking ingestion");
                client
                    .get_or_insert_ingestion(
                        &ingestion_uri,
                        &collection,
                        path.to_path_buf(),
                        languages.to_vec(),
                    )
                    .await?;

                println!("Starting crawl");
                ingestion_upload(
                    ingestion_uri,
                    &languages,
                    path,
                    bucket,
                    progress_reader,
                    format,
                )
                .await
            })()
            .await;

            CliResult::new(result, FailureExitCode::Upload).print_or_exit(format);
        }
        // Currently this command will only list up to 500 blobs,
        // due to restrictions in the Giant API.
        Commands::ListBlobs {
            giant_uri,
            collection,
            filter,
        } => {
            let mut client = GiantApiClient::new(giant_uri.clone());
            CliResult::new(
                client.get_blobs_in_collection(collection, filter).await,
                FailureExitCode::Api,
            )
            .print_or_exit(format);
        }
        Commands::DeleteCollection {
            giant_uri,
            collection,
        } => {
            let start_time = SystemTime::now();
            let result: Result<(), CliError> = (|| async {
                let mut client = GiantApiClient::new(giant_uri.clone());

                // Returns a maximum of 500 results,
                // so we need to loop until we've deleted them all.
                let mut blobs = client
                    .get_blobs_in_collection(collection, &ListBlobsFilter::All)
                    .await?;

                while !blobs.is_empty() {
                    for blob in blobs {
                        println!("Blob is in collections: {:?}", blob.collections);

                        let other_collections: Vec<String> = blob
                            .collections
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
                        let delete_blob_start_time = SystemTime::now();
                        client.delete_blob(&blob.uri).await?;
                        println!(
                            "Deleted blob {} in {}",
                            blob.uri,
                            format_duration(delete_blob_start_time.elapsed().unwrap())
                        );
                    }
                    blobs = client
                        .get_blobs_in_collection(collection, &ListBlobsFilter::All)
                        .await?;
                }

                println!("Deleting collection {}", collection);
                client.delete_collection(collection).await?;
                println!("Deleted collection {}", collection);

                Ok(())
            })()
            .await;

            // Incrementing a counter in an async context is surprisingly hard in Rust,
            // so we don't know how many blobs we deleted (for now, we can count the log lines).
            println!(
                "Deleted some blobs in {}",
                format_duration(start_time.elapsed().unwrap())
            );

            CliResult::new(result, FailureExitCode::Api).print_or_exit(format);
        }
    }
}
