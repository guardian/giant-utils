use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use futures::{stream, StreamExt};
use humantime::format_duration;
use indicatif::ProgressBar;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    model::{
        cli_error::CliError, file_metadata::FileMetadata, ingestion_file::IngestionFile,
        lang::Language, uri::Uri,
    },
    services::s3_client::S3Client,
};

pub async fn ingestion_upload(
    ingestion_uri: Uri,
    languages: &Vec<Language>,
    path: impl AsRef<Path>,
    bucket_name: &str,
    _sse_algorithm: &str,
) -> Result<(), CliError> {
    let s3_client = S3Client::new(bucket_name).await;

    println!("Counting files");
    // Not ideal to traverse twice but at least this way we are able to measure progress
    // Could experiment with spinning up two threads, one doing total counts and one doing uploads
    // This could potentially cause thrashing on a spinning magnet.
    let total_files = WalkDir::new(&path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| !f.path_is_symlink() && !f.file_type().is_dir())
        .count() as u64;

    // Do it again, this time logging failures to read files
    println!("Processing files");
    let walker = WalkDir::new(&path)
        .into_iter()
        .filter_map(|f| {
            if f.is_err() {
                // log
            }
            f.ok()
        })
        .filter(|f| !f.path_is_symlink() && !f.file_type().is_dir());

    let pb = ProgressBar::new(total_files);

    let start_time = SystemTime::now();
    let results = stream::iter(walker)
        .map(|dir| {
            let pb = &pb;
            let ingestion_uri = &ingestion_uri;
            let path = &path;
            let languages = &languages;
            let s3_client = &s3_client;

            async move {
                let current_millis = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let uuid = Uuid::new_v4();

                const DATA_PREFIX: &str = "data";
                const METADATA_PREFIX: &str = "metadata";
                const DATA_SUFFIX: &str = "data";
                const METADATA_SUFFIX: &str = "metadata.json";

                let ingestion_file = IngestionFile::from_file(ingestion_uri, &path, &dir).unwrap();
                let metadata =
                    FileMetadata::new(ingestion_uri, ingestion_file, languages, &dir.path());
                let metadata_key =
                    format!("{METADATA_PREFIX}/{current_millis}_{uuid}.{METADATA_SUFFIX}");
                if let Err(e) = s3_client.upload_metadata(&metadata_key, metadata).await {
                    eprintln!("Oh nO! {}", e);
                    pb.inc(1);
                    Err(e)?
                } else {
                    let data_key = format!("{DATA_PREFIX}/{current_millis}_{uuid}.{DATA_SUFFIX}");
                    if let Err(e) = s3_client.upload_file(&data_key, &dir.path()).await {
                        eprintln!("Oh nO! {}", e);
                        pb.inc(1);
                        Err(e)?
                    } else {
                        pb.inc(1);
                        Ok(())
                    }
                }
            }
        })
        .buffer_unordered(128)
        .collect::<Vec<anyhow::Result<()>>>()
        .await;

    let success_count = results
        .iter()
        .filter(|status| matches!(status, Ok(())))
        .count();
    let failure_count = results.len() - success_count;

    println!("Finished!");
    println!(
        "  Elapsed: {}",
        format_duration(start_time.elapsed().unwrap())
    );
    println!("  Success: {success_count}");
    println!("  Failure: {failure_count}");

    Ok(())
}
