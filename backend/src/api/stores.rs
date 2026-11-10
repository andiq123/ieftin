use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::AppState;
use crate::{
    crawler::{Activity, HEALTH_INTERVAL},
    domain::{Health, StoreOverview},
    error::AppResult,
    repo,
};

/// A store's stored state plus what is happening to it right now and what comes next.
#[derive(Serialize)]
pub struct StoreView {
    #[serde(flatten)]
    store: StoreOverview,
    activity: Activity,
    /// Estimated seconds until the running crawl finishes.
    eta_seconds: Option<i64>,
    next_check_at: Option<DateTime<Utc>>,
    /// Only healthy indexed stores are crawled.
    next_crawl_at: Option<DateTime<Utc>>,
}

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<StoreView>>> {
    let stores = repo::stores_overview(&state.pool).await?;
    let views = stores.into_iter().map(|store| {
        let activity = state.monitor.get(&store.id);
        let healthy = store.health_status == Some(Health::Ok);
        StoreView {
            eta_seconds: activity.eta_seconds(),
            next_check_at: store.health_checked_at.map(|checked| checked + HEALTH_INTERVAL),
            next_crawl_at: store.last_crawl_finished_at.filter(|_| healthy).map(|done| done + state.crawl_interval),
            activity,
            store,
        }
    });
    Ok(Json(views.collect()))
}
