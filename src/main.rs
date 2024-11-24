use anyhow::{Context, Result};
use axum::{Extension, Router};
use freshrss_image_cache_service_rs::app_service::{AppService, CloudFlareBypassProxy};
use freshrss_image_cache_service_rs::handlers::root_router;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("APP_PORT").with_context(|| "APP_PORT")?;
    let access_token = std::env::var("APP_ACCESS_TOKEN").with_context(|| "APP_ACCESS_TOKEN")?;
    let images_dir = std::env::var("APP_IMAGES_DIR").with_context(|| "APP_IMAGES_DIR")?;
    let no_colors = std::env::var("APP_NO_ANSI_COLORS").is_err();

    let cloudflare_proxy = std::env::var("APP_CLOUDFLARE_PROXY").ok();
    let cloudflare_proxy_login = std::env::var("APP_CLOUDFLARE_PROXY_LOGIN").ok();
    let cloudflare_proxy_pass = std::env::var("APP_CLOUDFLARE_PROXY_PASS").ok();

    let bypass_info = match (cloudflare_proxy, cloudflare_proxy_login, cloudflare_proxy_pass) {
        (Some(proxy), Some(login), Some(pass)) => Some(CloudFlareBypassProxy {
            proxy_url: proxy,
            proxy_login: login,
            proxy_password: pass,
        }),
        _ => None,
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(no_colors)
        .init();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Listening on http://{}", listener.local_addr()?);

    let app_service = Arc::new(AppService::new(access_token.clone(),
                                               PathBuf::from(images_dir.clone()),
                                               false, bypass_info.clone()));
    let safe = Router::new()
        .nest("/", root_router())
        .layer(Extension(app_service));

    let unsafe_app = Arc::new(AppService::new(access_token.clone(),
                                              PathBuf::from(images_dir.clone()),
                                              true, bypass_info));
    let unsafe_router = Router::new()
        .nest("/", root_router())
        .layer(Extension(unsafe_app));

    let app = Router::new()
        .nest("/notls", unsafe_router)
        .nest("/", safe);

    axum::serve(listener, app).await?;

    Ok(())
}
