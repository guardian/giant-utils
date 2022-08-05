use std::path::PathBuf;

use serde::Serialize;

use crate::model::lang::Language;

#[derive(Serialize)]
pub struct CreateIngestion {
    pub path: Option<PathBuf>,
    pub name: Option<String>,
    pub languages: Vec<Language>,
    pub fixed: Option<bool>,
    pub default: Option<bool>,
}
