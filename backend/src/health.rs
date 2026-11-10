//! Whether each store can be searched right now. Every store is tested with a known
//! query when the server starts and regularly after that; one that fails is left out
//! of searches until it passes again. Nothing is stored: a restart simply re-tests.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    fetch::FetchError,
    live::{LiveError, LiveSearch},
};

/// How often a working store is tested again.
const RECHECK_HEALTHY: Duration = Duration::from_secs(6 * 3600);
/// A failing store is retried sooner, so it returns to searches soon after it recovers.
const RECHECK_FAILING: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Prices can be read.
    Ok,
    /// The store refuses automated access (HTTP 401/403/429 or robots.txt).
    Blocked,
    /// The store could not be reached.
    Unreachable,
    /// The search page loads, but no readable price was found.
    NoData,
}

/// The outcome of a store's latest test; empty until the first one finishes.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub status: Option<Status>,
    pub detail: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    /// How long the store took to answer the test, when it answered.
    pub latency_ms: Option<u64>,
    /// A test is running right now.
    pub checking: bool,
}

impl Report {
    /// Untested stores are searched too: most work, and the test is seconds away.
    pub fn usable(&self) -> bool {
        matches!(self.status, None | Some(Status::Ok))
    }

    pub fn next_check_at(&self) -> Option<DateTime<Utc>> {
        Some(self.checked_at? + recheck_after(self.status?))
    }
}

/// Every store's latest report, shared between the checks and the API.
#[derive(Clone, Default)]
pub struct Health(Arc<RwLock<HashMap<&'static str, Report>>>);

impl Health {
    pub fn of(&self, store: &str) -> Report {
        self.0.read().unwrap_or_else(|poisoned| poisoned.into_inner()).get(store).cloned().unwrap_or_default()
    }

    fn update(&self, store: &'static str, change: impl FnOnce(&mut Report)) {
        change(self.0.write().unwrap_or_else(|poisoned| poisoned.into_inner()).entry(store).or_default());
    }
}

/// Starts one background task per store that tests it now and then on a schedule.
pub fn spawn_checks(live: Arc<LiveSearch>, health: &Health) {
    for store in live.stores().map(|store| store.id()).collect::<Vec<_>>() {
        let (live, health) = (live.clone(), health.clone());
        tokio::spawn(async move {
            loop {
                health.update(store, |report| report.checking = true);
                let started = Instant::now();
                let outcome = live.probe(store).await;
                let latency_ms = outcome.is_ok().then(|| started.elapsed().as_millis() as u64);
                let (status, detail) = match outcome {
                    Ok(0) => (Status::NoData, "search answered, no price found".to_owned()),
                    Ok(offers) => (Status::Ok, format!("{offers} offers read from a test search")),
                    Err(error) => (classify(&error), error.to_string()),
                };
                tracing::info!(store, ?status, "health check: {detail}");
                let report = Report {
                    status: Some(status),
                    detail: Some(detail),
                    checked_at: Some(Utc::now()),
                    latency_ms,
                    checking: false,
                };
                health.update(store, |current| *current = report);
                tokio::time::sleep(recheck_after(status)).await;
            }
        });
    }
}

fn recheck_after(status: Status) -> Duration {
    if status == Status::Ok { RECHECK_HEALTHY } else { RECHECK_FAILING }
}

fn classify(error: &LiveError) -> Status {
    const REFUSALS: [StatusCode; 3] = [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN, StatusCode::TOO_MANY_REQUESTS];
    match error {
        LiveError::Fetch(FetchError::Robots) => Status::Blocked,
        LiveError::Fetch(FetchError::Status(status)) if REFUSALS.contains(status) => Status::Blocked,
        _ => Status::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_untested_and_working_stores_only() {
        let report = |status| Report { status, ..Report::default() };
        assert!(report(None).usable());
        assert!(report(Some(Status::Ok)).usable());
        assert!(!report(Some(Status::Blocked)).usable());
        assert!(!report(Some(Status::NoData)).usable());
    }

    #[test]
    fn tells_refusals_from_outages() {
        assert_eq!(classify(&LiveError::Fetch(FetchError::Robots)), Status::Blocked);
        assert_eq!(classify(&LiveError::Fetch(FetchError::Status(StatusCode::FORBIDDEN))), Status::Blocked);
        assert_eq!(classify(&LiveError::Fetch(FetchError::Status(StatusCode::BAD_GATEWAY))), Status::Unreachable);
        assert_eq!(classify(&LiveError::Timeout), Status::Unreachable);
    }

    #[test]
    fn retries_failing_stores_sooner() {
        assert!(recheck_after(Status::Blocked) < recheck_after(Status::Ok));
    }
}
