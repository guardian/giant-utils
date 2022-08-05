use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;
use walkdir::DirEntry;

use super::uri::Uri;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestionFile {
    pub uri: Uri,
    pub parent_uri: Uri,
    pub size: u64,
    pub last_access_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
    pub creation_time: Option<DateTime<Utc>>,
    pub is_regular_file: bool,
}

impl IngestionFile {
    pub fn from_file(
        ingestion_uri: &Uri,
        base_path: impl AsRef<Path>,
        e: &DirEntry,
    ) -> anyhow::Result<IngestionFile> {
        let metadata = e.metadata()?;
        let relative_path = e.path().strip_prefix(base_path)?;

        let uri = ingestion_uri.extend_from_path(relative_path);

        let parent_uri: Uri = match relative_path.parent() {
            Some(parent_path) if parent_path.as_os_str() != "" => {
                ingestion_uri.extend_from_path(parent_path)
            }
            _ => ingestion_uri.clone(),
        };

        Ok(IngestionFile {
            uri,
            parent_uri,
            size: metadata.len() as u64,
            last_access_time: Some(metadata.accessed()?.into()),
            last_modified_time: Some(metadata.modified()?.into()),
            creation_time: Some(metadata.created()?.into()),
            is_regular_file: true,
        })
    }
}
