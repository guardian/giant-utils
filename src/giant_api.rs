use reqwest::{blocking::Client, header::HeaderMap, StatusCode};

use crate::{
    auth_store::{self},
    model::{cli_error::CliError, uri::Uri, forms::create_collection::CreateCollection, collection::Collection},
};

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

// pub fn check_ingestion_exists(uri: &str, id: &Uri) -> Result<bool, CliError> {
//     let collection = id.collection();
//     let mut url = String::from(uri);
//     url.push_str("/api/collections/");
//     url.push_str(collection);

//     let client = get_client(uri)?;

//     let res = client.get(url).send()?;
//     let status = res.status();

//     if status == 401 {
//         Err(CliError::APIAuthError)
//     } else {
//         let status = res.status();
//         println!("{}",res.text().unwrap());
//         Ok(status == 200)
//     }
// }

pub fn get_or_insert_collection(uri: &str, ingestion_uri: &Uri) -> Result<Collection, CliError> {
    let collection = ingestion_uri.collection();
    let url = format!("{uri}/api/collections/{collection}");

    let client = get_client(uri)?;

    let res = client.get(url).send()?;
    let status = res.status();
    
    if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::APIAuthError);
    } 

    if status == StatusCode::NOT_FOUND {
        // Insert collection
       let create_collection = CreateCollection { name: collection.to_owned() };
       let url = format!("{uri}/api/collections");
       let res = client.post(url).json(&create_collection).send()?;
       let status = res.status();

       if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::APIAuthError);
        } else if status != StatusCode::CREATED {
            return Err(CliError::UnexpectedResponse(status));
        }
        Ok(res.json::<Collection>()?)
        }
    else {
        Ok(res.json::<Collection>()?)
    }
}