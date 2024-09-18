use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::error;

#[derive(Debug)]
pub enum HttpError {
    Internal(anyhow::Error),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let HttpError::Internal(error) = self;

        error!(?error);

        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
