mod api;
mod config;
mod crawler;
mod domain;
mod error;
mod live;
mod repo;
mod search;
mod stores;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, live::LiveSearch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("connecting to the database")?;
    sqlx::migrate!().run(&pool).await.context("running migrations")?;

    let registry = stores::registry();
    repo::sync_stores(&pool, &registry).await?;
    let monitor = crawler::Monitor::default();
    let live = Arc::new(LiveSearch::new(registry.live, config.crawl.delay)?);
    if config.crawl.enabled {
        crawler::spawn(pool.clone(), registry.indexed, &config.crawl, &monitor)?;
        crawler::spawn_live_checks(pool.clone(), live.clone(), &monitor);
    }

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on http://{}", config.bind_addr);
    let state = api::AppState { pool, monitor, live, crawl_interval: config.crawl.interval };
    let app = api::router(state, config.cors_origins);
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
