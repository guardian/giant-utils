// This is a weird metadata class which we punt over to Giant so that the
// graph of disks can be implemented out of order
// If I was doing this again I don't think this would exist...

use std::path::Path;

use serde::Serialize;

use super::{ingestion_file::IngestionFile, lang::Language, uri::Uri};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    file: IngestionFile,
    ingestion: String, // the *FULL URI* for the ingestion
    languages: Vec<Language>,
}

impl FileMetadata {
    pub fn new(
        ingestion_uri: &Uri,
        file: IngestionFile,
        languages: &[Language],
        _path: impl AsRef<Path>,
    ) -> Self {
        FileMetadata {
            ingestion: ingestion_uri.as_str().to_owned(),
            file,
            languages: languages.to_vec(),
        }
    }
}
