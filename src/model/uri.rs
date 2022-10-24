use std::path::Path;

use super::cli_error::CliError;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Uri(String);

impl Uri {
    pub fn parse(uri: &str) -> Result<Uri, CliError> {
        let regex = Regex::new(r"[^\n^/.]+/([^\n^/.]+/?)+").unwrap();
        if regex.is_match(uri) {
            Ok(Uri(uri.to_owned()))
        } else {
            Err(CliError::InputError(format!(
                "URI must be in the form 'collection/ingestion'. Provided '{}'",
                uri
            )))
        }
    }

    pub fn extend_from_path(&self, path: impl AsRef<Path>) -> Uri {
        let base = self.0.clone();
        // Kinda annoying that we have to deal with encoding issues in paths which requires a lot of allocation
        let path_str = path.as_ref().display().to_string();
        let uri = if path_str.starts_with('/') {
            // This should generally not happen unless
            // you're ingesting from the root of your machine
            format!("{}{}", base, path_str.trim_end_matches('/'))
        } else {
            format!("{}/{}", base, path_str.trim_end_matches('/'))
        };

        Uri(uri)
    }

    pub fn collection(&self) -> &str {
        self.0.split('/').collect::<Vec<&str>>()[0]
    }

    pub fn ingestion(&self) -> &str {
        self.0.split('/').collect::<Vec<&str>>()[1]
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Uri {
    fn from(s: &str) -> Self {
        Uri(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // This is more restrictive than it should be to represent
    // all Giant URIs, as collection, blob and email URIs can be of depth 1.
    fn rejects_collection_uri() {
        let uri = "collection";
        let parsed_uri = Uri::parse(uri);
        assert!(
            matches!(parsed_uri, Err(_)),
            "Checking if '{}' parsing is Ok(()), was {:?} ",
            uri,
            parsed_uri
        );
    }

    #[test]
    fn accepts_ingestion_uri() {
        let uri = "collection/ingestion";
        let parsed_uri = Uri::parse(uri);
        assert!(
            matches!(parsed_uri, Ok(_)),
            "Checking if '{}' parsing is Ok(()), was {:?} ",
            uri,
            parsed_uri
        );
    }

    #[test]
    fn accepts_file_uri() {
        let uri = "collection/ingestion/directory/file";
        let parsed_uri = Uri::parse(uri);
        assert!(
            matches!(parsed_uri, Ok(_)),
            "Checking if '{}' parsing is Ok(()), was {:?} ",
            uri,
            parsed_uri
        );
    }
}
