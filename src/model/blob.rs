use reflection::Reflection;
use reflection_derive::Reflection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Reflection)]
pub struct Blob {
    pub uri: String,
    pub ingestions: Vec<String>,
    pub collections: Vec<String>,
}

#[derive(Deserialize)]
pub struct BlobResp {
    pub blobs: Vec<Blob>,
}
