use reflection::Reflection;
use reflection_derive::Reflection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Reflection)]
pub struct Blob {
    pub uri: String,
    #[serde(rename = "ingestion")]
    pub ingestions: Vec<String>,
}

#[derive(Deserialize)]
pub struct BlobResp {
    pub blobs: Vec<Blob>,
}
