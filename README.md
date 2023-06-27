# Resize S3 Images on the fly via Rust Lambda
This is a re-implementation of deprecated node.js lambda https://github.com/amazon-archives/serverless-image-resizing in Rust.

Check this doc for info how to setup it.


While this codebase is a working solution a guide needs to be written.
Meanwhile you can follow this [Guide](https://aws.amazon.com/blogs/compute/resize-images-on-the-fly-with-amazon-s3-aws-lambda-and-amazon-api-gateway/). FYI, format of s3 redirect has been changed.
Deployment was tested via [cargo lambda](https://github.com/cargo-lambda/cargo-lambda)
