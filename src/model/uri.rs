use super::cli_error::CliError;
use regex::Regex;

pub struct Uri(String);

impl Uri {
    pub fn parse(uri: &str) -> Result<Uri, CliError> {
        let regex = Regex::new(r"[^\n^/.]+/[^\n^/.]+").unwrap();
        if regex.is_match(uri) {
            Ok(Uri(uri.to_owned()))
        } else {
            Err(CliError::InputError(format!("URI must be in the form 'collection/ingestion'. Provided '{}'", uri)))
        }
    }

    pub fn collection(&self) -> &str {
        self.0.split('/').collect::<Vec<&str>>().first().unwrap()
    }
}