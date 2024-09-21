use crate::dtos::CachedImage;
use anyhow::{bail, Result};
use sha3::{Digest, Sha3_256};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Clone)]
pub struct CachedImagePaths {
    data_path: PathBuf,
    mime_type_path: PathBuf,
}

pub struct AppService {
    access_token: String,
    images_dir: PathBuf,
}

impl AppService {
    pub fn new(access_token: String, images_dir: PathBuf) -> Self {
        Self {
            access_token,
            images_dir,
        }
    }

    pub async fn get_image(&self, image_url: &Url) -> Result<CachedImage> {
        let cached_image_paths = self.get_file_name_path(image_url)?;
        let mut extracted_from_cache = true;

        if !cached_image_paths.data_path.as_path().exists() {
            self.download_and_save(image_url, cached_image_paths.clone())
                .await?;

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

        Ok(CachedImagePaths {
            data_path: self.images_dir.join(image_content_file_name),
            mime_type_path: self.images_dir.join(mime_type_file_name),
        })
    }

    async fn download_and_save(
        &self,
        image_url: &Url,
        cached_image_paths: CachedImagePaths,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let image_response = client.get(image_url.clone()).send().await?;

        let Some(mime_type) = image_response.headers().get("content-type").cloned() else {
            bail!("Failed to get the content-type header for image: {image_url}");
        };

        // 1. Save the image content
        let mut image_file = fs::File::create(cached_image_paths.data_path).await?;
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
}
