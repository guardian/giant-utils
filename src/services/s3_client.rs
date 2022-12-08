use std::path::Path;

use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_s3::{config, types::ByteStream, Client, Endpoint, Region};
use aws_smithy_http::body::SdkBody;

use crate::model::file_metadata::FileMetadata;

pub struct S3Client {
    client: Client,
    bucket_name: String,
}

impl S3Client {
    pub async fn new(bucket_name: &str) -> Self {
        let region_provider = Region::new("eu-west-1");

        let credentials_provider = DefaultCredentialsChain::builder()
            .region(region_provider)
            .build()
            .await;

        let shared_config = aws_config::from_env()
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let client = Client::new(&shared_config);

        S3Client {
            client,
            bucket_name: bucket_name.to_owned(),
        }
    }

    pub async fn from_endpoint(endpoint: http::Uri, bucket_name: &str) -> Self {
        let region_provider = Region::new("eu-west-1");

        let credentials_provider = DefaultCredentialsChain::builder()
            .region(region_provider)
            .build()
            .await;

        let shared_config = aws_config::from_env()
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let endpoint = Endpoint::immutable(endpoint);

        let s3_config = config::Builder::from(&shared_config)
            .endpoint_resolver(endpoint)
            .build();
        let client = Client::from_conf(s3_config);

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
