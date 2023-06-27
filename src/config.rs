use anyhow::{Context, Result};
use url::Url;

pub struct Bucket(pub String);

impl Bucket {
    pub fn load() -> Result<Self> {
        let bucket = std::env::var("BUCKET").context("env var BUCKET not found")?;
        tracing::debug!("Bucket is {}", &bucket);
        Ok(Self(bucket))
    }
}

pub fn get_base_url() -> Result<Url> {
    let base_url = std::env::var("URL").context("env var URL not found")?;
    tracing::debug!("URL is {}", base_url);
    let base_url =
        Url::parse(&base_url).context("Please set URL to proper http url - i.e. https://bucket")?;

    Ok(base_url)
}
