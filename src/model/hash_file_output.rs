use reflection::Reflection;
use reflection_derive::Reflection;
use serde::Serialize;

#[derive(Serialize, Reflection)]
pub struct HashFileOutput {
    pub hash: String,
    pub path: String,
}

impl HashFileOutput {
    pub fn new(hash: String, path: String) -> Self {
        HashFileOutput {
            hash, path
        }
    }
}