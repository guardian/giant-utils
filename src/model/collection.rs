use serde::{Deserialize, Serialize};

use super::ingestion::Ingestion;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub uri: String,
    pub display: String,
    pub ingestions: Vec<Ingestion>,
    pub created_by: Option<String>,
}
