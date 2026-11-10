use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::AppState;
use crate::health::Report;

/// A store and how its latest test went.
#[derive(Serialize)]
pub struct StoreView {
    id: &'static str,
    name: &'static str,
    url: &'static str,
    #[serde(flatten)]
    report: Report,
    next_check_at: Option<DateTime<Utc>>,
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<StoreView>> {
    let views = state.live.stores().map(|store| {
        let report = state.health.of(store.id());
        StoreView {
            id: store.id(),
            name: store.name(),
            url: store.home(),
            next_check_at: report.next_check_at(),
            report,
        }
    });
    Json(views.collect())
}
