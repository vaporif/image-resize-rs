use anyhow::{Context, Result};
use aws_config::SdkConfig;
use aws_sdk_s3::{operation::get_object::GetObjectError, primitives::ByteStream, Client};

use crate::{config::Bucket, error::Error};

pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new(sdk_config: &SdkConfig, bucket: Bucket) -> Self {
        let client = aws_sdk_s3::client::Client::new(sdk_config);
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
    pub async fn upload_image_bytes<I: Into<String> + std::fmt::Debug>(
        &self,
        image_key: I,
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
    use core::panic;

    use aws_config::{ConfigLoader, SdkConfig};
    use aws_sdk_s3::{config::Credentials, operation::create_bucket::CreateBucketError};

    use crate::{aws::S3Client, config::Bucket};

    async fn get_config() -> (SdkConfig, Bucket) {
        let credentials =
            Credentials::new("test".to_string(), "test".to_string(), None, None, "local");

        let test_config = ConfigLoader::default()
            // .endpoint_url("http://localhost:4566")
            .region("us-east-1")
            .credentials_provider(credentials)
            .load()
            .await;
        (test_config, Bucket("test".into()))
    }

    async fn ensure_bucket_exists() {
        let (config, bucket) = get_config().await;

        let client = aws_sdk_s3::client::Client::new(&config);

        let _r = client
            .list_buckets()
            .send()
            .await
            .expect("list buckets")
            .buckets()
            .expect("buckets found")
            .len();

        if let Err(CreateBucketError::Unhandled(unhandled_error)) = client
            .create_bucket()
            .bucket(bucket.0)
            .send()
            .await
            .map_err(|e| e.into_service_error())
        {
            panic!("failed to create bucket, error - {:?}", unhandled_error);
        }
    }

    // TODO: Find out why minio/localstack are not working
    #[ignore]
    #[tokio::test]
    async fn test_upload_and_download() {
        ensure_bucket_exists().await;

        let (config, bucket) = get_config().await;

        let s3_client = S3Client::new(&config, bucket).await;

        let image_key = "random_key".to_owned();
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
