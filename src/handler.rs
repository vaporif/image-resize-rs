use crate::{aws::S3Client, error::Error, request::ImageResizeRequest};
use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage};
use lambda_http::Request;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use url::Url;

pub async fn handle_resize(
    event: Request,
    base_url: &Url,
    s3_client: &S3Client,
) -> Result<Url, Error> {
    let resize_request = ImageResizeRequest::try_from(event.uri())?;
    tracing::info!("request: {}", resize_request);
    let image_bytes = s3_client.get_image_bytes(&resize_request.image_key).await?;
    let resized_bytes = resize(
        &image_bytes,
        resize_request.resolution.width,
        resize_request.resolution.height,
    )
    .await?;

    let new_image_path = format!(
        "{width}x{heigth}/{image_key}",
        width = resize_request.resolution.width,
        heigth = resize_request.resolution.height,
        image_key = resize_request.image_key
    );

    s3_client
        .upload_image_bytes(&new_image_path, resized_bytes)
        .await?;

    let new_image_url = base_url.join(&new_image_path).context(format!(
        "could not join base url with image url : {}",
        new_image_path
    ))?;

    tracing::info!("new image url is {}", &new_image_url);

    Ok(new_image_url)
}

#[tracing::instrument(skip(bytes))]
async fn resize(bytes: &[u8], nwidth: u16, nheight: u16) -> Result<Vec<u8>> {
    let format = image::guess_format(bytes).context("could not guess format")?;
    tracing::info!("format guessed as {:?}", format);
    let image = image::load_from_memory(bytes).context("failed to load image from memory")?;
    tracing::info!("loaded into memory");
    let image = image.resize(
        nwidth.into(),
        nheight.into(),
        get_filter_type(&image, nwidth.into(), nheight.into()),
    );
    tracing::info!("resized");
    let mut result_image_cursor = Cursor::new(Vec::new());
    image
        .write_to(&mut result_image_cursor, format)
        .context("should save result image")?;

    result_image_cursor.set_position(0);

    let mut bytes = Vec::new();
    result_image_cursor
        .read_to_end(&mut bytes)
        .await
        .context("should get all bytes of resized image")?;
    tracing::info!("result image saved into memory");
    Ok(bytes)
}

fn get_filter_type(image: &DynamicImage, nwidth: u32, nheight: u32) -> FilterType {
    let (cwidth, cheight) = (image.width(), image.height());

    if nwidth * nheight < cwidth * cheight {
        FilterType::CatmullRom
    } else {
        FilterType::Lanczos3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_eq;

    #[tokio::test]
    async fn test_resize() {
        let (nwidth, nheight) = (200, 125);
        let file_bytes = std::fs::read("samples/file.png").expect("file opened and bytes read");

        let resized_file_path = "samples/resized_file.png";
        let image_bytes = resize(&file_bytes, nwidth, nheight)
            .await
            .expect("should resizes");

        std::fs::write(resized_file_path, image_bytes).expect("resized image");

        let resized_file = image::open(resized_file_path).expect("resized file opened");
        let (width, heigth) = (resized_file.width(), resized_file.height());
        assert_eq!((nwidth as u32, nheight as u32), (width, heigth));
    }
}
