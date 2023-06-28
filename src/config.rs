use anyhow::{Context, Result};
use url::Url;

const BUCKET_ENV_VAR: &str = "BUCKET";
const URL_ENV_VAR: &str = "URL";

pub struct Bucket(pub String);

impl Bucket {
    pub fn load() -> Result<Self> {
        let bucket = std::env::var(BUCKET_ENV_VAR)
            .context(format!("env var {} not found", BUCKET_ENV_VAR))?;
        tracing::info!("Bucket is {}", &bucket);
        Ok(Self(bucket))
    }
}

pub fn get_base_url() -> Result<Url> {
    let base_url =
        std::env::var(URL_ENV_VAR).context(format!("env var {} not found", URL_ENV_VAR))?;
    tracing::info!("URL is {}", base_url);
    let base_url = Url::parse(&base_url).context(format!(
        "please set {} to proper http url - i.e. https://bucket",
        URL_ENV_VAR
    ))?;

    Ok(base_url)
}
