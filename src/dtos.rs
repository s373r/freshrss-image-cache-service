use crate::http_error::HttpError;
use axum::body::Body;
use axum::http::{header, HeaderName, HeaderValue};
use axum::response::IntoResponse;
use std::str::FromStr;

pub struct CachedImage {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub extracted_from_cache: bool,
}

impl IntoResponse for CachedImage {
    fn into_response(self) -> axum::response::Response {
        let content_type_value = match HeaderValue::from_str(&self.mime_type) {
            Ok(value) => value,
            Err(e) => {
                let wrapped_error = HttpError::Internal(anyhow::Error::new(e));

                return wrapped_error.into_response();
            }
        };

        let mut response = Body::from(self.data).into_response();
        let headers = response.headers_mut();

        let cache_status_value = if self.extracted_from_cache {
            "HIT"
        } else {
            "MISS"
        };

        headers.insert(header::CONTENT_TYPE, content_type_value);
        headers.insert(
            HeaderName::from_str("X-Piccache-Status").unwrap(),
            HeaderValue::from_str(cache_status_value).unwrap(),
        );

        response
    }
}
