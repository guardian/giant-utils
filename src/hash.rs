use std::{
    fs::File,
    io::{BufReader, Read},
};

use sha2::{Digest, Sha512};

use crate::model::{cli_error::CliError, hash_file_output::HashFileOutput};

pub fn hash_file(path: String) -> Result<HashFileOutput, CliError> {
    let f = File::open(&path)?;
    let mut reader = BufReader::new(f);

    let mut hasher = Sha512::new();

    let mut buf = [0u8; 512];

    loop {
        let byte_count = reader.read(&mut buf)?;

        if byte_count == 0 {
            break;
        }

        hasher.update(&buf[..byte_count]);
    }

    let digest = hasher.finalize();

    let encoded_digest = base64::encode_config(digest, base64::URL_SAFE_NO_PAD);

    Ok(HashFileOutput::new(encoded_digest, path))
}
