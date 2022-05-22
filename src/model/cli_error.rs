use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    #[error("Request Error")]
    Request(#[from] reqwest::Error),
    #[error("Header error")]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("API Auth Error")]
    APIAuthError,
    #[error("Your current OS is not supported, please use Linux, MacOS, or Windows")]
    UnsupportedSystem,
}