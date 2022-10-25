use std::path::PathBuf;

use clap::ValueEnum;
use reqwest::{header::HeaderMap, Client, Error, StatusCode, Url};
use reqwest::{RequestBuilder, Response};

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
        let auth_token = auth_store::get(base_url.as_str()).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", auth_token.parse().unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self { client, base_url }
    }

    async fn send_request(&mut self, request_builder: RequestBuilder) -> Result<Response, Error> {
        let resp = request_builder.send().await?;
        let auth_response_header = resp.headers().get("X-Offer-Authorization");

        match auth_response_header {
            Some(token_header_value) => {
                let token = token_header_value
                    .to_str()
                    .expect("X-Offer-Authorization should contain only ASCII chars");
                println!("Giant API returned new token in X-Offer-Authorization header. Refreshing client and auth store");
                auth_store::set(self.base_url.as_str(), token).unwrap();
                let mut headers = HeaderMap::new();
                headers.insert("Authorization", token_header_value.clone());
                self.client = Client::builder().default_headers(headers).build().unwrap();
            }
            None => println!("No X-Offer-Authorization header in response from Giant API"),
        }

        Ok(resp)
    }

    pub async fn check_hash_exists(&mut self, hash: &str) -> Result<bool, CliError> {
        let mut url = self.base_url.clone();

        url.path_segments_mut()
            .unwrap()
            .push("api")
            .push("resources")
            .push(hash);

        url.query_pairs_mut().append_pair("basic", "true");

        let res = self.send_request(self.client.get(url)).await?;
        let status = res.status();

        if status == 401 {
            Err(CliError::APIAuthError)
        } else {
            Ok(res.status() == 200)
        }
    }

    pub async fn get_or_insert_collection(
        &mut self,
        ingestion_uri: &Uri,
    ) -> Result<Collection, CliError> {
        let collection = ingestion_uri.collection();

        let mut collections_url = self.base_url.clone();
        collections_url
            .path_segments_mut()
            .unwrap()
            .push("api")
            .push("collections");

        let mut collection_url = collections_url.clone();
        collection_url.path_segments_mut().unwrap().push(collection);

        let res = self.send_request(self.client.get(collection_url)).await?;
        let status = res.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(CliError::APIAuthError);
        }

        if status == StatusCode::NOT_FOUND {
            // Insert collection
            let create_collection = CreateCollection {
                name: collection.to_owned(),
            };
            let res = self
                .send_request(self.client.post(collections_url).json(&create_collection))
                .await?;
            let status = res.status();

            if status == StatusCode::UNAUTHORIZED {
                return Err(CliError::APIAuthError);
            } else if status != StatusCode::CREATED {
                return Err(CliError::UnexpectedResponse(status));
            }
            Ok(res.json::<Collection>().await?)
        } else {
            Ok(res.json::<Collection>().await?)
        }
    }

    pub async fn get_or_insert_ingestion(
        &mut self,
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

            let res = self
                .send_request(self.client.post(url).json(&create_ingestion))
                .await?;
            let status = res.status();

            if status == StatusCode::OK {
                Ok(())
            } else {
                Err(CliError::UnexpectedResponse(status))
            }
        }
    }

    // Returns a maximum of 500 blobs per request
    pub async fn get_blobs_in_collection(
        &mut self,
        collection: &str,
        filter: &ListBlobsFilter,
    ) -> Result<Vec<Blob>, CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().push("api").push("blobs");

        let in_multiple = match filter {
            ListBlobsFilter::InMultiple => "true",
            ListBlobsFilter::All => "false",
        };

        url.query_pairs_mut().append_pair("inMultiple", in_multiple);
        url.query_pairs_mut().append_pair("collection", collection);

        let res = self.send_request(self.client.get(url)).await?;
        let status = res.status();

        if status == StatusCode::OK {
            let resp = res.json::<BlobResp>().await?;
            Ok(resp.blobs)
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }

    pub async fn delete_blob(&mut self, blob_uri: &str) -> Result<(), CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("api")
            .push("blobs")
            .push(blob_uri);

        // checkChildren=false means we'll delete blobs even if they have
        // children (i.e. because they're archives and contain further files).
        url.query_pairs_mut().append_pair("checkChildren", "false");

        let res = self.send_request(self.client.delete(url)).await?;

        let status = res.status();

        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }

    pub async fn delete_collection(&mut self, collection: &str) -> Result<(), CliError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .unwrap()
            .push("api")
            .push("collections")
            .push(collection);

        let res = self.send_request(self.client.delete(url)).await?;
        let status = res.status();

        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(CliError::UnexpectedResponse(status))
        }
    }
}
