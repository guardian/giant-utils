use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_smithy_http::{operation::Response, result::SdkError};
use reqwest::{header::InvalidHeaderValue, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    // We need to see the underlying request error in stderr when the CLI
    // fails, otherwise we have no idea what happened.
    // https://docs.rs/thiserror/latest/thiserror/
    #[error(transparent)]
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
