use aws_sdk_s3::{error::PutObjectError, types::SdkError};
use aws_smithy_http::operation::Response;
use reqwest::{header::InvalidHeaderValue, StatusCode};
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
    #[error("Input error: {0}")]
    InputError(String),
    #[error("Unexpected response from server: {0}")]
    UnexpectedResponse(StatusCode),
    #[error("Error while uploading to S3")]
    IngestionUploadError(#[from] Box<SdkError<PutObjectError, Response>>),
    #[error("JSON error")]
    JsonError(#[from] serde_json::Error),
}
