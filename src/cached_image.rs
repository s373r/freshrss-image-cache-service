use crate::http_error::HttpError;
use axum::body::Body;
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;

pub struct CachedImage {
    pub data: Vec<u8>,
    pub mime_type: String,
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

        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type_value);

        response
    }
}
