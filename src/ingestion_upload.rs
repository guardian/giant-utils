use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use futures::{stream, StreamExt};
use humantime::format_duration;
use indicatif::ProgressBar;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    model::{
        cli_error::CliError,
        cli_output::OutputFormat,
        file_metadata::FileMetadata,
        ingestion_file::IngestionFile,
        lang::Language,
        log_message::{FailureStage, LogMessage},
        uri::Uri,
    },
    services::s3_client::S3Client,
};

pub async fn ingestion_upload(
    ingestion_uri: Uri,
    languages: &Vec<Language>,
    path: impl AsRef<Path>,
    bucket_name: &str,
    format: &OutputFormat,
) -> Result<(), CliError> {
    let s3_client = S3Client::new(bucket_name).await;

    let (sender, mut receiver) = mpsc::unbounded_channel::<LogMessage>();

    // Slightly annoying clone so we can move the format into the background worker
    let format = format.clone();
    tokio::spawn(async move {
        let log_name = format!("{}_ingestion.log", Utc::now().to_rfc3339());
        let log_file = tokio::fs::File::create(log_name)
            .await
            .expect("Failed to create log file");
        let mut writer = BufWriter::new(log_file);

        match format {
            OutputFormat::JSON => {
                while let Some(message) = receiver.recv().await {
                    let buf = message.to_json();
                    writer.write_all(buf.as_bytes()).await.unwrap();
                }
            }
            OutputFormat::TSV => {
                while let Some(message) = receiver.recv().await {
                    let buf = message.to_tsv_row();
                    writer.write_all(buf.as_bytes()).await.unwrap()
                }
            }
        }
    });

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
            let log_sender = sender.clone();

            async move {
                let start = SystemTime::now();
                let start_millis = start.duration_since(UNIX_EPOCH).unwrap().as_millis();

                let uuid = Uuid::new_v4();

                const DATA_PREFIX: &str = "data";
                const METADATA_PREFIX: &str = "metadata";
                const DATA_SUFFIX: &str = "data";
                const METADATA_SUFFIX: &str = "metadata.json";

                let file_size = dir.metadata()?.len();
                let ingestion_file = IngestionFile::from_file(ingestion_uri, &path, &dir).unwrap();
                let metadata = FileMetadata::new(ingestion_uri, ingestion_file, languages);
                let metadata_key =
                    format!("{METADATA_PREFIX}/{start_millis}_{uuid}.{METADATA_SUFFIX}");
                if let Err(e) = s3_client.upload_metadata(&metadata_key, metadata).await {
                    eprintln!("Failure in ingestion pipeline: {}", e);
                    log_sender.send(LogMessage::Failure {
                        path: path.as_ref().to_owned(),
                        size: file_size,
                        start_millis,
                        end_millis: start.duration_since(UNIX_EPOCH).unwrap().as_millis(),
                        failure_stage: FailureStage::UploadMetadata,
                        reason: e.to_string(),
                    })?;
                    pb.inc(1);
                    Err(e)?
                } else {
                    let data_key = format!("{DATA_PREFIX}/{start_millis}_{uuid}.{DATA_SUFFIX}");
                    if let Err(e) = s3_client.upload_file(&data_key, &dir.path()).await {
                        eprintln!("Failure in ingestion pipeline: {}", e);
                        log_sender.send(LogMessage::Failure {
                            path: path.as_ref().to_owned(),
                            size: file_size,
                            start_millis,
                            end_millis: start.duration_since(UNIX_EPOCH).unwrap().as_millis(),
                            failure_stage: FailureStage::UploadData,
                            reason: e.to_string(),
                        })?;
                        pb.inc(1);
                        Err(e)?
                    } else {
                        log_sender.send(LogMessage::Success {
                            path: path.as_ref().to_owned(),
                            size: file_size,
                            start_millis,
                            end_millis: start.duration_since(UNIX_EPOCH).unwrap().as_millis(),
                        })?;
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
