use anyhow::{Context, Result};
use aws_sdk_s3::{operation::get_object::GetObjectError, primitives::ByteStream, Client, Config};

use crate::{config::Bucket, error::Error};

pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub fn new(config: Config, bucket: Bucket) -> Self {
        let client = aws_sdk_s3::client::Client::from_conf(config);
        Self {
            client,
            bucket: bucket.0,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_image_bytes(&self, image_key: &str) -> Result<Vec<u8>, Error> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(image_key)
            .send()
            .await
            .map_err(|err| match err.into_service_error() {
                GetObjectError::NoSuchKey(f) => Error::NotFound(f),
                f => anyhow::Error::new(f).into(),
            })?;

        let bytes = object
            .body
            .collect()
            .await
            .context("failed to load image data")?
            .into_bytes();

        tracing::info!("image downloaded");
        Ok(bytes.into())
    }

    #[tracing::instrument(skip(self, bytes))]
    pub async fn upload_image_bytes(
        &self,
        image_key: impl Into<String> + std::fmt::Debug,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let stream = ByteStream::from(bytes);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(image_key)
            .body(stream)
            .send()
            .await
            .context("failed to upload image")?;

        tracing::info!("image uploaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use aws_config::ConfigLoader;
    use aws_sdk_s3::{config::Credentials, Config};

    use crate::{aws::S3Client, config::Bucket};

    async fn get_config() -> (Config, Bucket) {
        let credentials =
            Credentials::new("test".to_string(), "test".to_string(), None, None, "local");

        let test_config = ConfigLoader::default()
            .behavior_version(aws_config::BehaviorVersion::v2023_11_09())
            .endpoint_url("http://localhost:4566")
            .region("us-east-1")
            .credentials_provider(credentials)
            .load()
            .await;

        let test_config = aws_sdk_s3::config::Builder::from(&test_config)
            .force_path_style(true)
            .build();

        (test_config, Bucket("bucket".to_string()))
    }

    #[tokio::test]
    async fn test_upload_and_download() {
        let (config, bucket) = get_config().await;

        let s3_client = S3Client::new(config, bucket);

        let image_key = "random_key".to_string();
        let bytes = vec![1, 3, 4];

        s3_client
            .upload_image_bytes(&image_key, bytes.clone())
            .await
            .expect("upload_success");

        let downloaded_bytes = s3_client
            .get_image_bytes(&image_key)
            .await
            .expect("download success");

        assert!(bytes == downloaded_bytes);
    }
}
