use std::path::PathBuf;

use clap::ValueEnum;
use reqwest::{blocking::Client, Error, header::HeaderMap, Method, StatusCode, Url};
use reqwest::blocking::Response;

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
use crate::model::blob::{Blob, BlobResp};

#[derive(ValueEnum, Clone)]
pub enum ListBlobsFilter {
    All,
    InMultiple,
}

pub struct GiantApiClient {
    client: Client,
    base_url: Url,
}

impl GiantApiClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            client: {
                let auth_token = auth_store::get(base_url.as_str()).unwrap();
                let mut headers = HeaderMap::new();
                headers.insert("Authorization", auth_token.parse().unwrap());
                Client::builder().default_headers(headers).build().unwrap()
            },
            base_url,
        }
    }

    fn request(&mut self, method: Method, url: Url) -> Result<Response, Error> {
        let resp = self.client.request(method, url).send()?;
        let auth_response_header = resp.headers().get("X-Offer-Authorization");

        match auth_response_header {
            Some(token_header_value) => {
                let token = token_header_value.to_str().expect("X-Offer-Authorization should contain only ASCII chars");
                println!("Giant API returned new token in X-Offer-Authorization header. Refreshing client and auth store");
                auth_store::set(self.base_url.as_str(), token).unwrap();
                let mut headers = HeaderMap::new();
                headers.insert("Authorization", token_header_value.clone());
                self.client = Client::builder().default_headers(headers).build().unwrap();
            }
            None => println!("No X-Offer-Authorization header in response from Giant API")
        }

        Ok(resp)
    }

    pub fn check_hash_exists(&mut self, hash: &str) -> Result<bool, CliError> {
        let mut url = self.base_url.clone();

        url.path_segments_mut()
            .unwrap()
            .push("api")
            .push("resources")
            .push(hash);

        url.query_pairs_mut()
            .append_pair("basic", "true");

        let res = self.request(Method::GET, url)?;
        let status = res.status();

        if status == 401 {
            Err(CliError::APIAuthError)
        } else {
            Ok(res.status() == 200)
        }
    }

    pub fn get_or_insert_collection(&self, ingestion_uri: &Uri) -> Result<Collection, CliError> {
        let collection = ingestion_uri.collection();

        let mut collections_url = self.base_url.clone();
        collections_url.path_segments_mut()
            .unwrap()
            .push("api")
            .push("collections");

        let mut collection_url = collections_url.clone();
        collection_url.path_segments_mut().unwrap().push(collection);

        let res = self.client.get(collection_url).send()?;
        let status = res.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(CliError::APIAuthError);
        }

        if status == StatusCode::NOT_FOUND {
            // Insert collection
            let create_collection = CreateCollection {
                name: collection.to_owned(),
            };
            let res = self.client.post(collections_url).json(&create_collection).send()?;
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
        &self,
        ingestion_uri: &Uri,
        base_collection: &Collection,
        path: PathBuf,
        languages: Vec<Language>,
    ) -> Result<(), CliError> {
        let mut url = self.base_url.clone();

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
            url.path_segments_mut()
                .unwrap()
                .push("api")
                .push("collections")
                .push(collection);

            let create_ingestion = CreateIngestion {
                path: Some(path),
                name: Some(ingestion.to_owned()),
                languages,
                fixed: Some(false), // This is hardcoded to false in the existing CLI
                default: Some(false),
            };

            let res = self.client.post(url).json(&create_ingestion).send()?;
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
        &self,
        collection: &str,
        filter: &ListBlobsFilter,
    ) -> Result<Vec<Blob>, CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap()
            .push("api")
            .push("blobs");

        let in_multiple = match filter {
            ListBlobsFilter::InMultiple => "true",
            ListBlobsFilter::All => "false",
        };

        url.query_pairs_mut().append_pair("inMultiple", in_multiple);
        url.query_pairs_mut().append_pair("collection", collection);

        let res = self.client.get(url).send()?;
        let status = res.status();

        if status == StatusCode::OK {
            let resp = res.json::<BlobResp>()?;
            Ok(resp.blobs)
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }

    pub fn delete_blob(&self, blob_uri: &str) -> Result<(), CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap()
            .push("api")
            .push("blobs")
            .push(blob_uri);

        // checkChildren=false means we'll delete blobs even if they have
        // children (i.e. because they're archives and contain further files).
        url.query_pairs_mut().append_pair("checkChildren", "false");

        let res = self.client.delete(url).send()?;

        let status = res.status();

        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }

    pub fn delete_collection(&self, collection: &str) -> Result<(), CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap()
            .push("api")
            .push("collections")
            .push(collection);

        let res = self.client.delete(url).send()?;
        let status = res.status();

        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }
}
