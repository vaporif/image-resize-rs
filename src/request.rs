use crate::error::Error;
use anyhow::Context;
use lambda_http::http::Uri;
use regex::Regex;
use serde::Deserialize;

const RESOLUTIONS_ENV_VAR: &str = "RESOLUTIONS";

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
}

#[derive(PartialEq, Eq, Debug)]
pub struct ImageResizeRequest {
    pub image_key: String,
    pub resolution: Resolution,
}

impl std::fmt::Display for ImageResizeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "requested {} with {}x{}",
            self.image_key, self.resolution.width, self.resolution.height
        )
    }
}

impl TryFrom<&Uri> for ImageResizeRequest {
    type Error = Error;

    fn try_from(uri: &Uri) -> Result<Self, Self::Error> {
        let regex = Regex::new(r"/((\d+)x(\d+))/(.*)")
            .context("failed to parse image resize request regex")?;
        let path = &uri.path().to_string();
        let caps = regex.captures(path)
            .context("could not get image key with requested dimensions from http path - expected http path format is /{width}x{height}/image.png")?;

        let image_key = caps
            .get(4)
            .context("image_key not in url path")?
            .as_str()
            .into();

        let width = caps
            .get(2)
            .context("width not in url path")?
            .as_str()
            .parse()
            .context("width should be number")?;

        let height = caps
            .get(3)
            .context("height not in url path")?
            .as_str()
            .parse()
            .context("height should be number")?;

        let request = ImageResizeRequest {
            image_key,
            resolution: Resolution { width, height },
        };

        if let Some(resolutions) = get_permitted_resolutions() {
            let resolution_found = resolutions
                .into_iter()
                .any(|supported_res| supported_res == request.resolution);

            if !resolution_found {
                return Err(Error::UnsupportedResolution);
            }
        }

        Ok(request)
    }
}

fn get_permitted_resolutions() -> Option<Vec<Resolution>> {
    let resolutions = std::env::var(RESOLUTIONS_ENV_VAR);
    if let Ok(resolutions) = resolutions {
        match serde_json::from_str::<Vec<Resolution>>(&resolutions) {
            Ok(v) => {
                tracing::info!("Got supported resolutions of {:?}", resolutions);
                Some(v)
            }
            Err(e) => {
                dbg!(&e);
                tracing::error!(
                    "env var {} with value {} has failed to deserialize: {}",
                    RESOLUTIONS_ENV_VAR,
                    resolutions,
                    e
                );
                tracing::error!(
                    "HINT: try format `export RESOLUTIONS=[{{\"width\":100,\"height\":300}}]`"
                );

                None
            }
        }
    } else {
        tracing::warn!(
            "env var {} not set! All resolutions are available for resize!!!",
            RESOLUTIONS_ENV_VAR
        );
        None
    }
}

#[cfg(test)]
mod tests {
    use std::matches;

    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn when_proper_path_and_no_resolutions_limit() {
        let url: Uri = Uri::from_static("https://some-domain.com/200x300/key.png");
        let image_request = ImageResizeRequest::try_from(&url).expect("url parsed");

        assert_eq!(
            image_request,
            ImageResizeRequest {
                image_key: "key.png".into(),
                resolution: Resolution {
                    width: 200,
                    height: 300
                }
            }
        );
    }

    #[test]
    #[serial]
    fn when_proper_path_and_supported_resolution() {
        std::env::set_var("RESOLUTIONS", r#"[{"width":200,"height":300}]"#);
        let url: Uri = Uri::from_static("https://some-domain.com/200x300/key.png");
        let image_request = ImageResizeRequest::try_from(&url).expect("url parsed");

        assert_eq!(
            image_request,
            ImageResizeRequest {
                image_key: "key.png".into(),
                resolution: Resolution {
                    width: 200,
                    height: 300
                }
            }
        );
    }

    #[test]
    #[serial]
    fn when_proper_path_and_unsupported_resolution() {
        std::env::set_var("RESOLUTIONS", r#"[{"width":400,"height":500}]"#);
        let url: Uri = Uri::from_static("https://some-domain.com/200x300/key.png");
        let image_request_result = ImageResizeRequest::try_from(&url);

        assert!(matches!(
            image_request_result,
            Err(Error::UnsupportedResolution),
        ));
    }

    #[test]
    #[serial]
    fn when_not_proper_path() {
        let url: Uri = Uri::from_static("https://some-domain.com/2srs00x310srs0/key.png");

        assert!(ImageResizeRequest::try_from(&url).is_err());
    }
}
