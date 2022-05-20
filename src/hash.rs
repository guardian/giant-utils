use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use sha2::{Digest, Sha512};

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let f = File::open(path)?;
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

    let encoded_digest = base64::encode_config(digest, base64::STANDARD_NO_PAD);

    Ok(encoded_digest)
}
