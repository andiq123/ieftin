use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Utc};
use serde::Serialize;

/// What a store's background task is doing right now.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Activity {
    #[default]
    Idle,
    Checking,
    Crawling {
        started_at: DateTime<Utc>,
        pages_done: usize,
        /// Pages discovered and not yet read. Grows while sitemaps are still loading.
        pages_pending: usize,
        offers: usize,
    },
}

impl Activity {
    /// Seconds left at the pace so far; `None` until there is a pace to measure.
    pub fn eta_seconds(&self) -> Option<i64> {
        let Self::Crawling { started_at, pages_done, pages_pending, .. } = *self else { return None };
        let elapsed = (Utc::now() - started_at).num_seconds();
        (pages_done >= 10).then(|| elapsed * pages_pending as i64 / pages_done as i64)
    }
}

/// Live, in-memory view of every store's background task, shared with the API.
/// It is deliberately not persisted: after a restart nothing is running.
#[derive(Clone, Default)]
pub struct Monitor(Arc<RwLock<HashMap<&'static str, Activity>>>);

impl Monitor {
    pub fn set(&self, store: &'static str, activity: Activity) {
        self.0.write().unwrap_or_else(|poisoned| poisoned.into_inner()).insert(store, activity);
    }

    pub fn get(&self, store: &str) -> Activity {
        self.0.read().unwrap_or_else(|poisoned| poisoned.into_inner()).get(store).copied().unwrap_or_default()
    }
}
