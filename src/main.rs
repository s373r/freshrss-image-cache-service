use anyhow::{Context, Result};
use axum::{Extension, Router};
use freshrss_image_cache_service_rs::app_service::AppService;
use freshrss_image_cache_service_rs::handlers::root_router;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("APP_PORT").with_context(|| "APP_PORT")?;
    let access_token = std::env::var("APP_ACCESS_TOKEN").with_context(|| "APP_ACCESS_TOKEN")?;
    let images_dir = std::env::var("APP_IMAGES_DIR").with_context(|| "APP_IMAGES_DIR")?;
    let no_colors = std::env::var("APP_NO_ANSI_COLORS").is_err();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(no_colors)
        .init();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Listening on http://{}", listener.local_addr()?);

    let app_service = Arc::new(AppService::new(access_token, PathBuf::from(images_dir)));
    let app = Router::new()
        .nest("/", root_router())
        .layer(Extension(app_service));

    axum::serve(listener, app).await?;

    Ok(())
}
