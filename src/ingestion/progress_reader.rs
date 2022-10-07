use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use flashmap::{self, ReadHandle};

use crate::model::{cli_error::CliError, log_message::LogMessage};

pub type ProgressReader = ReadHandle<PathBuf, bool>;

pub fn empty_progress_reader() -> ProgressReader {
    let (_, read) = flashmap::new::<PathBuf, bool>();
    read
}

pub fn progress_reader_from_path(path: impl AsRef<Path>) -> Result<ProgressReader, CliError> {
    let (mut write, read) = flashmap::new::<PathBuf, bool>();

    let mut write_guard = write.guard();
    match path.as_ref().extension().and_then(|e| e.to_str()) {
        Some("tsv") => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;

                let mut cols = line.split('\t');
                let status = cols
                    .next()
                    .ok_or_else(|| CliError::InputError("Invalid column in log file".into()))?;
                let path = cols
                    .next()
                    .ok_or_else(|| CliError::InputError("Invalid column in log file".into()))?;

                if status == "success" {
                    write_guard.insert(PathBuf::from(path), true);
                }
            }
        }
        Some("ndjson") => {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let log_entry = serde_json::from_str(&line)?;

                if let LogMessage::Failure { path, .. } = log_entry {
                    write_guard.insert(path, true);
                }
            }
        }
        _ => {}
    }

    write_guard.publish();

    Ok(read)
}
