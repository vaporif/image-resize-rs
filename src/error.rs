use aws_sdk_s3::types::error::NoSuchKey;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(NoSuchKey),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
