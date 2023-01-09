use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Arabic,
    English,
    French,
    German,
    Russian,
    Portuguese,
    Persian,
}
