use aws::S3Client;
use config::{get_base_url, Bucket};
use handler::handle_resize;
use lambda_http::{run, Body, Error, Request, Response};
use lambda_runtime::tower::ServiceBuilder;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use url::Url;

mod aws;
mod config;
mod error;
mod handler;
mod request;

async fn function_handler(
    event: Request,
    base_url: &Url,
    s3_client: &S3Client,
) -> Result<Response<Body>, Error> {
    match handle_resize(event, base_url, s3_client).await {
        Ok(url) => Ok(Response::builder()
            .status(301)
            .header("location", url.to_string())
            .body("".into())
            .map_err(Box::new)?),
        Err(e) => match e {
            error::Error::NotFound(e) => {
                tracing::error!("Image not found, {}", e);
                Ok(Response::builder()
                    .status(404)
                    .body("".into())
                    .map_err(Box::new)?)
            }
            other => Err(Error::from(other)),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let aws_config = aws_config::load_from_env().await;

    let bucket = Bucket::load()?;
    let s3_client = S3Client::new(&aws_config, bucket).await;

    let base_url = get_base_url()?;

    let layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    let service = ServiceBuilder::new()
        .layer(layer)
        .service_fn(|req| function_handler(req, &base_url, &s3_client));
    run(service).await?;
    Ok(())
}
