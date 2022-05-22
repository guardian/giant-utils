use std::error::Error;

use clap::{ ArgEnum};
use reflection::Reflection;
use serde::Serialize;
use tsv::Config;

use super::exit_code::ExitCode;

#[derive(ArgEnum, Clone)]
pub enum OutputFormat {
    TSV,
    JSON,
}

pub struct CliResult<T: Serialize + Reflection, E: Error> {
    inner: Result<T, E>,
    exit_code: ExitCode,
}

impl<T: Serialize + Reflection, E: Error> CliResult<T, E> { 
    pub fn new(inner: Result<T, E>, exit_code: ExitCode) -> Self {
        CliResult {inner, exit_code}
    }

    pub fn exit(self)  {
        match self.inner {
            Ok(_) => {}, 
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(self.exit_code as i32);
            }
        }
    }

    pub fn print_or_exit(self, format: &OutputFormat) {
        match self.inner {
            Ok(r) => match format {
                OutputFormat::TSV  => {
                    match Config::make_config(false, "()".into(), "TRUE".into(), "FALSE".into()) {
                        Ok(config) => {
                            match tsv::to_string(&r, config) {
                                Ok(text) => println!("{}", text),
                                Err(e) => {
                                    eprintln!("Failed to serialize output");
                                    eprintln!("{}", e);
                                    std::process::exit(ExitCode::SerializationFailed as i32);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Invalid TSV output config, you'll need a new build of this tool to fix this");
                            eprintln!("{}", e);
                            std::process::exit(ExitCode::SerializationFailed as i32);
                        },
                    }
                },
                OutputFormat::JSON  => {
                    match serde_json::to_string(&r) {
                        Ok(text) => println!("{}", text),
                        Err(e) => {
                            eprintln!("Failed to serialize output");
                            eprintln!("{}", e);
                            std::process::exit(ExitCode::SerializationFailed as i32);
                        }
                    }
                },
            }, 
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(self.exit_code as i32);
            }
        }
    }
}

