use clap::ValueEnum;
use serde::{Serialize, Deserialize};

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    English,
    French,
}