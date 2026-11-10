mod api;
mod config;
mod domain;
mod error;
mod fetch;
mod health;
mod live;
mod search;
mod stores;

use std::{net::SocketAddr, sync::Arc};

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, health::Health, live::LiveSearch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let live = Arc::new(LiveSearch::new(stores::all())?);
    let health = Health::default();
    health::spawn_checks(live.clone(), &health);

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on http://{}", config.bind_addr);
    let app = api::router(api::AppState { live, health }, config.cors_origins);
    // The search rate limit needs each connection's address.
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_requested())
        .await?;
    Ok(())
}

/// Resolves on Ctrl+C, or on the SIGTERM a container runtime sends to stop the app.
async fn shutdown_requested() {
    let interrupt = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => tokio::select! {
                _ = interrupt => {}
                _ = terminate.recv() => {}
            },
            Err(_) => drop(interrupt.await),
        }
    }
    #[cfg(not(unix))]
    drop(interrupt.await);
}
