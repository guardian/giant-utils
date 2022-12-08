//! Utility functions for dealing with AWS

use aws_config::default_provider::credentials::DefaultCredentialsChain;

pub async fn build_credentials_provider(profile: Option<String>) -> DefaultCredentialsChain {
    let mut builder = DefaultCredentialsChain::builder();

    if let Some(profile) = profile {
        builder = builder.profile_name(&profile);
    }

    builder.build().await
}
