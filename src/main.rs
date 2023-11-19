use aws::S3Client;
use config::get_configs;
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
        Err(e) => {
            tracing::error!("{:?}", e);
            match e {
                error::Error::NotFound(_) => Ok(Response::builder()
                    .status(404)
                    .body("".into())
                    .map_err(Box::new)?),
                error::Error::UnsupportedResolution => Ok(Response::builder()
                    .status(400)
                    .body("Unsupported resolution".into())
                    .map_err(Box::new)?),
                _ => Ok(Response::builder()
                    .status(500)
                    .body("Internal Server Error".into())
                    .map_err(Box::new)?),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let (config, bucket, base_url) = get_configs().await?;
    let s3_client = S3Client::new(config, bucket);

    let layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    let service = ServiceBuilder::new()
        .layer(layer)
        .service_fn(|req| function_handler(req, &base_url, &s3_client));
    run(service).await?;
    Ok(())
}
