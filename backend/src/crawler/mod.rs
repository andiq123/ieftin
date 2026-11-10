//! Background work, one independent task per store: a daily health check for every
//! store, and for indexed stores a full crawl on a fixed interval while healthy.

mod fetcher;
mod health;
mod monitor;
mod robots;
mod walk;

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use self::walk::Walk;
pub use self::{
    fetcher::{FetchError, Fetcher},
    monitor::{Activity, Monitor},
};
use crate::{
    config::CrawlConfig,
    domain::Health,
    live::{LiveError, LiveSearch},
    repo,
    stores::Store,
};

/// Tries per request while crawling; nobody is waiting, so transient failures are retried.
const CRAWL_ATTEMPTS: u32 = 3;

pub const HEALTH_INTERVAL: Duration = Duration::from_secs(24 * 3600);
const RETRY_AFTER_FAILURE: Duration = Duration::from_secs(600);
/// A store that keeps failing mid-crawl has gone down; stop instead of hammering it.
const MAX_CONSECUTIVE_FAILURES: usize = 30;

struct Crawler {
    pool: PgPool,
    store: Arc<dyn Store>,
    fetcher: Fetcher,
    crawl_interval: Duration,
    monitor: Monitor,
}

/// Starts one independent background task per store.
pub fn spawn(pool: PgPool, stores: Vec<Arc<dyn Store>>, config: &CrawlConfig, monitor: &Monitor) -> Result<()> {
    for store in stores {
        let crawler = Crawler {
            pool: pool.clone(),
            fetcher: Fetcher::new(config.delay, CRAWL_ATTEMPTS)?,
            crawl_interval: config.interval,
            monitor: monitor.clone(),
            store,
        };
        tokio::spawn(async move { crawler.run().await });
    }
    Ok(())
}

/// Starts the daily health check of every live store: its probe query must return offers.
pub fn spawn_live_checks(pool: PgPool, live: Arc<LiveSearch>, monitor: &Monitor) {
    for store in live.store_ids().collect::<Vec<_>>() {
        let (pool, live, monitor) = (pool.clone(), live.clone(), monitor.clone());
        tokio::spawn(async move {
            loop {
                let pause = check_live(&pool, &live, &monitor, store).await.unwrap_or_else(|error| {
                    tracing::error!(store, "{error:#}");
                    RETRY_AFTER_FAILURE
                });
                monitor.set(store, Activity::Idle);
                tokio::time::sleep(pause).await;
            }
        });
    }
}

/// Checks the store if its last check is a day old; returns how long to wait.
async fn check_live(pool: &PgPool, live: &LiveSearch, monitor: &Monitor, store: &'static str) -> Result<Duration> {
    let due = due_in(repo::crawl_state(pool, store).await?.health_checked_at, HEALTH_INTERVAL);
    if !due.is_zero() {
        return Ok(due);
    }
    monitor.set(store, Activity::Checking);
    let (status, detail) = match live.probe(store).await {
        Ok(0) => (Health::NoData, "search answered, no price found".to_owned()),
        Ok(offers) => (Health::Ok, format!("{offers} offers read from a test search")),
        Err(LiveError::Fetch(error)) => (health::classify(&error), error.to_string()),
        Err(error) => (Health::Unreachable, error.to_string()),
    };
    tracing::info!(store, ?status, "health check: {detail}");
    repo::save_health(pool, store, status, &detail).await?;
    Ok(HEALTH_INTERVAL)
}

/// Time left until something done at `last` is due again; zero when it never ran.
fn due_in(last: Option<DateTime<Utc>>, interval: Duration) -> Duration {
    let Some(last) = last else {
        return Duration::ZERO;
    };
    let age = (Utc::now() - last).to_std().unwrap_or_default();
    interval.saturating_sub(age)
}

impl Crawler {
    async fn run(self) {
        loop {
            let pause = self.step().await.unwrap_or_else(|error| {
                tracing::error!(store = self.store.id(), "{error:#}");
                RETRY_AFTER_FAILURE
            });
            self.monitor.set(self.store.id(), Activity::Idle);
            tokio::time::sleep(pause).await;
        }
    }

    /// Does the one thing that is due, if any, and returns how long to wait before
    /// looking again.
    async fn step(&self) -> Result<Duration> {
        let state = repo::crawl_state(&self.pool, self.store.id()).await?;
        let health_due = due_in(state.health_checked_at, HEALTH_INTERVAL);
        if health_due.is_zero() {
            self.check_health().await?;
            return Ok(Duration::ZERO);
        }
        if state.health_status != Some(Health::Ok) {
            return Ok(health_due);
        }
        let crawl_due = due_in(state.last_crawl_finished_at, self.crawl_interval);
        if crawl_due.is_zero() {
            self.crawl().await?;
            return Ok(Duration::ZERO);
        }
        Ok(health_due.min(crawl_due))
    }

    async fn check_health(&self) -> Result<()> {
        let store = self.store.id();
        self.monitor.set(store, Activity::Checking);
        let (status, detail) = health::check(Walk::new(self.store.as_ref(), &self.fetcher)?.sampling()).await;
        tracing::info!(store, ?status, "health check: {detail}");
        repo::save_health(&self.pool, store, status, &detail).await?;
        Ok(())
    }

    async fn crawl(&self) -> Result<()> {
        let store = self.store.id();
        repo::mark_crawl_started(&self.pool, store).await?;
        tracing::info!(store, "crawl started");

        let started_at = Utc::now();
        let mut walk = Walk::new(self.store.as_ref(), &self.fetcher)?;
        let (mut offers, mut failures) = (0, 0);
        loop {
            let progress =
                Activity::Crawling { started_at, pages_done: walk.done(), pages_pending: walk.pending(), offers };
            self.monitor.set(store, progress);
            let Some(step) = walk.next().await else { break };
            match step {
                Ok(page) => {
                    failures = 0;
                    offers += repo::upsert_offers(&self.pool, store, &page.offers).await.context("saving offers")?;
                }
                Err(error) => {
                    failures += 1;
                    tracing::debug!(store, "page skipped: {error}");
                    if failures >= MAX_CONSECUTIVE_FAILURES {
                        // Re-diagnose now, so the store is not crawled again until it recovers.
                        self.check_health().await?;
                        bail!("crawl aborted after {failures} consecutive failures, last: {error}");
                    }
                }
            }
        }

        let pruned = repo::mark_crawl_finished(&self.pool, store).await?;
        tracing::info!(store, offers, pruned, "crawl finished");
        Ok(())
    }
}
