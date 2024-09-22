use crate::app_service::AppService;
use crate::http_error::HttpError;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

pub fn root_router() -> Router {
    Router::new().route("/", get(get_image_handler).post(proactive_cache_handler))
}

#[derive(Deserialize, Debug)]
struct ImageParams {
    url: Url,
}

#[tracing::instrument(level = "debug", skip_all, fields(%url))]
async fn get_image_handler(
    Extension(app_service): Extension<Arc<AppService>>,
    Query(ImageParams { url }): Query<ImageParams>,
) -> impl IntoResponse {
    app_service
        .get_image(&url)
        .await
        .map_err(HttpError::Internal)
}

#[derive(Deserialize, Debug)]
struct CacheProactiveImage {
    access_token: String,
    url: Url,
}

#[tracing::instrument(level = "debug", skip_all, fields(%payload.url))]
async fn proactive_cache_handler(
    Extension(app_service): Extension<Arc<AppService>>,
    Json(payload): Json<CacheProactiveImage>,
) -> impl IntoResponse {
    if !app_service.validate_access_token(&payload.access_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    if let Err(e) = app_service.save_image(&payload.url).await {
        return HttpError::Internal(e).into_response();
    }

    #[derive(Serialize)]
    struct Response {
        status: &'static str,
    }

    Json(Response { status: "OK" }).into_response()
}
