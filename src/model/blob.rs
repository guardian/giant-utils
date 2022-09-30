use serde::{Serialize, Deserialize};
use reflection::Reflection;
use reflection_derive::Reflection;

#[derive(Debug, Serialize, Deserialize, Reflection)]
pub struct Blob {
    pub uri: String,
    // TODO: change Giant API to pluralise this since it clearly returns an array
    pub ingestion: Vec<String>,
}

#[derive(Deserialize)]
pub struct BlobResp {
    pub blobs: Vec<Blob>
}