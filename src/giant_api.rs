use reqwest::{header::{HeaderMap}, blocking::Client};

use crate::{auth_store::{self}, model::cli_error::CliError};

pub fn get_client(uri: &str) -> Result<Client, CliError> {
    let auth_token = auth_store::get(uri)?;
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", auth_token.parse()?);

    Ok(Client::builder().default_headers(headers).build()?)
}

pub fn check_hash_exists(uri: &str, hash: &str) -> Result<bool, CliError> {
    let mut url = String::from(uri);
    url.push_str("/api/resources/");
    url.push_str(hash);
    url.push_str("?basic=true");

    let client = get_client(uri)?;

    let res = client.get(url).send()?;
    let status = res.status();    

    if status == 401 {
        Err(CliError::APIAuthError)
    } else {
        Ok(res.status() == 200)
    }

}