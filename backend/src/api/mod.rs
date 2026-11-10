mod history;
mod search;
mod stores;

use std::{sync::Arc, time::Duration};

use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{crawler::Monitor, live::LiveSearch};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub monitor: Monitor,
    pub live: Arc<LiveSearch>,
    pub crawl_interval: Duration,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/stores", get(stores::list))
        .route("/search", get(search::search))
        .route("/history", get(history::history));
    Router::new().nest("/api", api).layer(CompressionLayer::new()).layer(TraceLayer::new_for_http()).with_state(state)
}
