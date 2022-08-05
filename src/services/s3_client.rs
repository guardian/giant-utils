use std::path::Path;

use aws_sdk_s3::{types::ByteStream, Client, Region};
use aws_smithy_http::body::SdkBody;

use crate::model::file_metadata::FileMetadata;

pub struct S3Client {
    client: Client,
    bucket_name: String,
}

impl S3Client {
    pub async fn new(bucket_name: &str) -> Self {
        let region_provider = Region::new("eu-west-1");

        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        S3Client {
            client,
            bucket_name: bucket_name.to_owned(),
        }
    }

    pub async fn upload_file(&self, key: &str, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let body = ByteStream::from_path(path).await?;
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn upload_metadata(&self, key: &str, metadata: FileMetadata) -> anyhow::Result<()> {
        let json = serde_json::to_string(&metadata)?;
        let body = ByteStream::new(SdkBody::from(&*json));

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(body)
            .send()
            .await?;

        Ok(())
    }
}
