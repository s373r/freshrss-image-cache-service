use std::error::Error;
use std::fmt;
use crate::dtos::CachedImage;
use anyhow::{bail, Context, Result};
use sha3::{Digest, Sha3_256};
use std::path::PathBuf;
use axum::http::HeaderValue;
use reqwest::Response;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Clone)]
pub struct CachedImagePaths {
    data_folder: PathBuf,
    data_path: PathBuf,
    mime_type_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct CloudFlareBypassProxy {
    pub proxy_url: String,
    pub proxy_login: String,
    pub proxy_password: String,
}

pub struct AppService {
    access_token: String,
    images_dir: PathBuf,
    disable_https_validation: bool,
    bypass_info: Option<CloudFlareBypassProxy>,
}


#[derive(Debug)]
struct CaptchaError {
    mime_type: String,
}
impl Error for CaptchaError {}

impl fmt::Display for CaptchaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Received a CAPTCHA page, mime type: {}", self.mime_type)
    }
}

impl AppService {
    pub fn new(access_token: String, images_dir: PathBuf, disable_https_validation: bool,
        bypass_info: Option<CloudFlareBypassProxy>) -> Self {
        Self {
            access_token,
            images_dir,
            disable_https_validation,
            bypass_info,
        }
    }

    pub async fn get_image(&self, image_url: &Url) -> Result<CachedImage> {
        let cached_image_paths = self.get_file_name_path(image_url)?;
        let mut extracted_from_cache = true;

        if !cached_image_paths.data_path.as_path().exists() {
            self.download_and_save(image_url, cached_image_paths.clone())
                .await
                .with_context(|| "Failed to download and save an cached image")?;

            extracted_from_cache = false;
        }

        let (image_content, mime_type) = tokio::try_join!(
            fs::read(cached_image_paths.data_path),
            fs::read_to_string(cached_image_paths.mime_type_path)
        )?;

        Ok(CachedImage {
            data: image_content,
            mime_type,
            extracted_from_cache,
        })
    }

    pub async fn save_image(&self, image_url: &Url) -> Result<()> {
        let cached_image_paths = self.get_file_name_path(image_url)?;

        self.download_and_save(image_url, cached_image_paths).await
    }

    pub fn validate_access_token(&self, request_access_token: &str) -> bool {
        self.access_token == request_access_token
    }

    fn get_file_name_path(&self, image_url: &Url) -> Result<CachedImagePaths> {
        let image_content_file_name = {
            let mut hasher = Sha3_256::new();
            hasher.update(image_url.as_str());
            let hash = hasher.finalize();
            hex::encode(&hash[..])
        };
        let mime_type_file_name = format!("{image_content_file_name}.mime-type");
        // Use two first letters of the file name for the intermediate directory, to
        // make sure we don't end up with a huge number of files.
        let data_folder = self.images_dir.join(&image_content_file_name[0..2]);

        Ok(CachedImagePaths {
            data_path: data_folder.join(image_content_file_name),
            mime_type_path: data_folder.join(mime_type_file_name),
            data_folder,
        })
    }

    async fn download_and_save(
        &self,
        image_url: &Url,
        cached_image_paths: CachedImagePaths,
    ) -> Result<()> {
        // Make sure the directory for the file exists
        fs::create_dir_all(cached_image_paths.data_folder).await?;

        let mut res = self.try_get_image(image_url, false).await;
        if let Err(err) = res {
            // If not a CAPTCHA error or we can't bypass CloudFlare, return the error
            if !err.downcast_ref::<CaptchaError>().is_some() || self.bypass_info.is_none() {
                return Err(err);
            }

            // Try to use the CloudFlare bypass
            res = self.try_get_image(image_url, true).await;
            if let Err(err) = res {
                return Err(err);
            }
        }

        let (image_response, mime_type) = res?;

        // 1. Save the image content
        let mut image_file = fs::File::create(cached_image_paths.data_path.clone())
            .await
            .with_context(|| {
                format!(
                    "Failed to create the '{}' file",
                    cached_image_paths.data_path.display()
                )
            })?;
        let mut response_content_stream = image_response.bytes_stream();

        use futures_util::stream::StreamExt;

        while let Some(chunk) = response_content_stream.next().await {
            let chunk = chunk?;
            image_file.write_all(&chunk).await?;
        }

        // 2. Save the mime-type
        fs::write(cached_image_paths.mime_type_path, mime_type).await?;

        Ok(())
    }

    async fn try_get_image(&self, image_url: &Url, deploy_workarounds: bool) -> Result<(Response, HeaderValue)> {
        let mut client_builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.disable_https_validation);

        if deploy_workarounds {
            let bypass = self.bypass_info.as_ref().unwrap();
            let proxy = reqwest::Proxy::all(bypass.proxy_url.clone())
                .expect("Failed to create a proxy")
                .basic_auth(&bypass.proxy_login.clone(), &bypass.proxy_password.clone());
            client_builder =
                client_builder.danger_accept_invalid_certs(true).proxy(proxy);
        }

        let client = client_builder.build().expect("Failed to build the HTTP client");

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("user-agent", "curl/7.86.0".parse().unwrap());
        headers.insert("accept", "*/*".parse().unwrap());

        let image_response = client.get(image_url.clone()).headers(headers).send().await?;

        let Some(mime_type) = image_response.headers().get("content-type").cloned() else {
            bail!("Failed to get the content-type header for image: {image_url}");
        };

        // CloudFlare often returns CAPTCHAs for images, so we need to check for that.
        let mime_str = String::from_utf8_lossy(mime_type.as_bytes());
        if !mime_str.starts_with("image/") && !mime_str.starts_with("video/") {
            return Err(CaptchaError { mime_type: mime_str.parse()? }.into());
        }
        Ok((image_response, mime_type))
    }
}
