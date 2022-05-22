use std::{
    io::{ Read, Write},
    fs::{File, self}, path::PathBuf
};

use urlencoding::encode;

use crate::model::cli_error::CliError;

fn get_path(uri: &str) -> Result<PathBuf, CliError> {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".giant-utils");
        if !path.exists() {
            fs::create_dir(&path)?;
        }
        let encoded_uri = encode(uri);

        path.push(encoded_uri.as_ref());
        Ok(path)
    } else {
        Err(CliError::UnsupportedSystem)
    }
}

pub fn get(uri: &str) -> Result<String, CliError> {
    let path = get_path(uri)?;
    let mut file = File::open(path)?;

    let mut token = String::new();
    file.read_to_string(&mut token)?;

    Ok(token)
}

pub fn set(uri: &str, token: &str) -> Result<(), CliError>{
    let path = get_path(uri)?;
    let mut file = File::create(path)?;

    file.write_all(token.as_bytes()).map_err(|e| e.into())
}
