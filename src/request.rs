use anyhow::{Context, Result};
use regex::Regex;

#[derive(PartialEq, Eq, Debug)]
pub struct ImageResizeRequest {
    pub image_key: String,
    pub width: u16,
    pub height: u16,
}

impl std::fmt::Display for ImageResizeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "requested {} with {}x{}",
            self.image_key, self.width, self.height
        )
    }
}

pub fn parse(request_path: &str) -> Result<ImageResizeRequest> {
    let regex =
        Regex::new(r"/((\d+)x(\d+))/(.*)").context("failed to parse image resize request regex")?;
    let caps = regex.captures(request_path).context("could not get image key with requested dimensions from http path - expected http path format is /{width}x{height}/image.png")?;

    Ok(ImageResizeRequest {
        image_key: caps
            .get(4)
            .context("image_key not in url path")?
            .as_str()
            .into(),
        width: caps
            .get(2)
            .context("width not in url path")?
            .as_str()
            .parse()
            .context("width should be number")?,
        height: caps
            .get(3)
            .context("height not in url path")?
            .as_str()
            .parse()
            .context("height should be number")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_proper_path() {
        let request_path = String::from("dir/200x300/key");
        let image_request = parse(&request_path).expect("capture done");

        assert_eq!(
            image_request,
            ImageResizeRequest {
                image_key: "key".into(),
                width: 200,
                height: 300
            }
        );
    }

    #[test]
    fn when_not_proper_path() {
        let request_path = String::from("dir/20ts1300/keytt");
        assert!(parse(&request_path).is_err());
    }
}
