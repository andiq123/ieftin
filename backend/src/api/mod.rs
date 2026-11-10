mod limit;
mod search;
mod stores;

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::get,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

pub use self::limit::RateLimit;
use crate::{health::Health, live::LiveSearch};

#[derive(Clone)]
pub struct AppState {
    pub live: Arc<LiveSearch>,
    pub health: Health,
    pub limit: RateLimit,
}

/// `cors_origins` are the websites whose pages may call the API; `None` allows any.
pub fn router(state: AppState, cors_origins: Option<Vec<HeaderValue>>) -> Router {
    let origins = cors_origins.map_or_else(AllowOrigin::any, AllowOrigin::list);
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/stores", get(stores::list))
        .route("/search", get(search::search));
    Router::new()
        .nest("/api", api)
        .layer(CorsLayer::new().allow_origin(origins).allow_methods([Method::GET]))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
