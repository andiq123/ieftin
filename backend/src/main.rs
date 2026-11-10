mod api;
mod config;
mod crawler;
mod domain;
mod error;
mod live;
mod repo;
mod search;
mod stores;

use std::sync::Arc;

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
    axum::serve(listener, api::router(api::AppState { pool, monitor, live, crawl_interval: config.crawl.interval }))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
