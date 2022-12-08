use std::error::Error;

use clap::ValueEnum;
use reflection::Reflection;
use serde::Serialize;
use tsv::Config;

use super::exit_code::FailureExitCode;

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Tsv,
    Json,
}

impl OutputFormat {
    pub fn to_extension(&self) -> &'static str {
        match self {
            Self::Json => "ndjson",
            Self::Tsv => "tsv",
        }
    }
}

pub struct CliResult<T: Serialize + Reflection, E: Error> {
    inner: Result<T, E>,
    exit_code: FailureExitCode,
}

impl<T: Serialize + Reflection, E: Error> CliResult<T, E> {
    pub fn new(inner: Result<T, E>, exit_code: FailureExitCode) -> Self {
        CliResult { inner, exit_code }
    }

    pub fn exit(self) {
        match self.inner {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(self.exit_code as i32);
            }
        }
    }

    pub fn print_or_exit(self, format: &OutputFormat) {
        match self.inner {
            Ok(r) => match format {
                OutputFormat::Tsv => {
                    match Config::make_config(false, "()".into(), "TRUE".into(), "FALSE".into()) {
                        Ok(config) => match tsv::to_string(&r, config) {
                            Ok(text) => println!("{text}"),
                            Err(e) => {
                                eprintln!("Failed to serialize output");
                                eprintln!("{e}");
                                std::process::exit(FailureExitCode::Serialization as i32);
                            }
                        },
                        Err(e) => {
                            eprintln!("Invalid TSV output config, you'll need a new build of this tool to fix this");
                            eprintln!("{e}");
                            std::process::exit(FailureExitCode::Serialization as i32);
                        }
                    }
                }
                OutputFormat::Json => match serde_json::to_string(&r) {
                    Ok(text) => println!("{text}"),
                    Err(e) => {
                        eprintln!("Failed to serialize output");
                        eprintln!("{e}");
                        std::process::exit(FailureExitCode::Serialization as i32);
                    }
                },
            },
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(self.exit_code as i32);
            }
        }
    }
}
