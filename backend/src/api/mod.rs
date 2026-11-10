mod history;
mod limit;
mod search;
mod stores;

use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    http::{HeaderValue, Method},
    middleware,
    routing::get,
};
use sqlx::PgPool;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use self::limit::RateLimit;
use crate::{crawler::Monitor, live::LiveSearch};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub monitor: Monitor,
    pub live: Arc<LiveSearch>,
    pub crawl_interval: Duration,
}

/// `cors_origins` are the websites whose pages may call the API; `None` allows any.
pub fn router(state: AppState, cors_origins: Option<Vec<HeaderValue>>) -> Router {
    let origins = cors_origins.map_or_else(AllowOrigin::any, AllowOrigin::list);
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/stores", get(stores::list))
        .route(
            "/search",
            get(search::search).layer(middleware::from_fn_with_state(RateLimit::default(), limit::enforce)),
        )
        .route("/history", get(history::history));
    Router::new()
        .nest("/api", api)
        .layer(CorsLayer::new().allow_origin(origins).allow_methods([Method::GET]))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
