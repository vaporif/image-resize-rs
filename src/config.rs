use anyhow::{Context, Result};
use aws_sdk_s3::config::Region;
use url::Url;

const BUCKET_ENV_VAR: &str = "BUCKET";
const REGION_ENV_VAR: &str = "LAMBDA_REGION";
const URL_ENV_VAR: &str = "URL";

pub async fn get_configs() -> Result<(aws_sdk_s3::Config, Bucket, Url)> {
    let region = get_region()?;

    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::v2023_11_09())
        .region(region)
        .load()
        .await;
    let config = aws_sdk_s3::Config::new(&aws_config);

    let bucket = Bucket::load()?;
    let base_url = get_base_url()?;

    Ok((config, bucket, base_url))
}

pub struct Bucket(pub String);

impl Bucket {
    pub fn load() -> Result<Self> {
        let bucket = std::env::var(BUCKET_ENV_VAR)
            .context(format!("env var {} not found", BUCKET_ENV_VAR))?;
        tracing::info!("Bucket is {}", &bucket);
        Ok(Self(bucket))
    }
}

fn get_base_url() -> Result<Url> {
    let base_url =
        std::env::var(URL_ENV_VAR).context(format!("env var {} not found", URL_ENV_VAR))?;
    tracing::info!("URL is {}", base_url);
    let base_url = Url::parse(&base_url).context(format!(
        "please set {} to proper http url - i.e. https://bucket",
        URL_ENV_VAR
    ))?;

    Ok(base_url)
}

fn get_region() -> Result<Region> {
    let region =
        std::env::var(REGION_ENV_VAR).context(format!("env var {} not found", REGION_ENV_VAR))?;

    Ok(Region::new(region))
}
