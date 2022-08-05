use serde::{Deserialize, Serialize};

use super::lang::Language;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ingestion {
    pub display: String,
    pub uri: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub path: Option<String>,
    pub failure_message: Option<String>,
    pub languages: Vec<Language>,
    pub fixed: bool,
    pub default: bool,
}
