use std::path::PathBuf;

use reqwest::{blocking::Client, header::HeaderMap, StatusCode};

use crate::model::blob::{Blob, BlobResp};
use crate::{
    auth_store::{self},
    model::{
        cli_error::CliError,
        collection::Collection,
        forms::{create_collection::CreateCollection, create_ingestion::CreateIngestion},
        lang::Language,
        uri::Uri,
    },
};

use clap::ValueEnum;

use urlencoding::encode;

#[derive(ValueEnum, Clone)]
pub enum ListBlobsFilter {
    All,
    InMultiple,
}

pub fn get_client(giant_uri: &str) -> Result<Client, CliError> {
    let auth_token = auth_store::get(giant_uri)?;
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", auth_token.parse()?);

    Ok(Client::builder().default_headers(headers).build()?)
}

pub fn check_hash_exists(uri: &str, hash: &str) -> Result<bool, CliError> {
    let mut url = String::from(uri);
    url.push_str("/api/resources/");
    url.push_str(hash);
    url.push_str("?basic=true");

    let client = get_client(uri)?;

    let res = client.get(url).send()?;
    let status = res.status();

    if status == 401 {
        Err(CliError::APIAuthError)
    } else {
        Ok(res.status() == 200)
    }
}

pub fn get_or_insert_collection(uri: &str, ingestion_uri: &Uri) -> Result<Collection, CliError> {
    let collection = ingestion_uri.collection();
    let url = format!("{uri}/api/collections/{collection}");

    let client = get_client(uri)?;

    let res = client.get(url).send()?;
    let status = res.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::APIAuthError);
    }

    if status == StatusCode::NOT_FOUND {
        // Insert collection
        let create_collection = CreateCollection {
            name: collection.to_owned(),
        };
        let url = format!("{uri}/api/collections");
        let res = client.post(url).json(&create_collection).send()?;
        let status = res.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(CliError::APIAuthError);
        } else if status != StatusCode::CREATED {
            return Err(CliError::UnexpectedResponse(status));
        }
        Ok(res.json::<Collection>()?)
    } else {
        Ok(res.json::<Collection>()?)
    }
}

pub fn get_or_insert_ingestion(
    uri: &str,
    ingestion_uri: &Uri,
    base_collection: &Collection,
    path: PathBuf,
    languages: Vec<Language>,
) -> Result<(), CliError> {
    let collection = ingestion_uri.collection();
    let ingestion = ingestion_uri.ingestion();

    if base_collection
        .ingestions
        .iter()
        .any(|i| i.uri == ingestion_uri.as_str())
    {
        // collection already contains ingestion!
        Ok(())
    } else {
        let client = get_client(uri)?;
        let url = format!("{uri}/api/collections/{collection}");

        let create_ingestion = CreateIngestion {
            path: Some(path),
            name: Some(ingestion.to_owned()),
            languages,
            fixed: Some(false), // This is hardcoded to false in the existing CLI
            default: Some(false),
        };

        let res = client.post(url).json(&create_ingestion).send()?;
        let status = res.status();

        if status == StatusCode::OK {
            Ok(())
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }
}

// Returns a maximum of 500 blobs per request
pub fn get_blobs_in_collection(
    giant_uri: &str,
    collection: &str,
    filter: &ListBlobsFilter,
) -> Result<Vec<Blob>, CliError> {
    let client = get_client(giant_uri)?;
    let encoded_collection = encode(collection);
    let in_multiple = match filter {
        ListBlobsFilter::All => "",
        ListBlobsFilter::InMultiple => "&inMultiple=true",
    };
    let url = format!("{giant_uri}/api/blobs?collection={encoded_collection}{in_multiple}");
    let res = client.get(url).send()?;
    let status = res.status();

    if status == StatusCode::OK {
        let resp = res.json::<BlobResp>()?;
        Ok(resp.blobs)
    } else {
        Err(CliError::UnexpectedResponse(status))
    }
}

pub fn delete_blob(giant_uri: &str, blob_uri: &str) -> Result<(), CliError> {
    // TODO QUESTION: is it wasteful to keep getting this client over and over?
    let client = get_client(giant_uri)?;

    let encoded_blob_uri = encode(blob_uri);

    // checkChildren=false means we'll delete blobs even if they have
    // children (i.e. because they're archives and contain further files).
    let url = format!("{giant_uri}/api/blobs/{encoded_blob_uri}?checkChildren=false");

    let res = client.delete(url).send()?;

    let status = res.status();

    if status == StatusCode::NO_CONTENT {
        Ok(())
    } else {
        Err(CliError::UnexpectedResponse(status))
    }
}

pub fn delete_collection(giant_uri: &str, collection: &str) -> Result<(), CliError> {
    let client = get_client(giant_uri)?;
    let encoded_collection = encode(collection);
    let url = format!("{giant_uri}/api/collections/{encoded_collection}");
    let res = client.delete(url).send()?;
    let status = res.status();

    if status == StatusCode::NO_CONTENT {
        Ok(())
    } else {
        Err(CliError::UnexpectedResponse(status))
    }
}
