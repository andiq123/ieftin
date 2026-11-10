//! Searching: asks each store's own search page for the query and reads the results,
//! with a short-lived cache so paging and repeated searches cost the stores nothing.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use futures::future::join_all;
use serde::Serialize;

use crate::{
    domain::ScrapedOffer,
    fetch::{FetchError, Fetcher},
    stores::Store,
};

/// A search waits this long for a store before answering without it.
const TIMEOUT: Duration = Duration::from_secs(7);
/// Minimum time between two requests to the same store.
const PAUSE: Duration = Duration::from_millis(300);
/// How long a store's answer to a query is reused. Prices rarely move within minutes.
const CACHE_TTL: Duration = Duration::from_secs(600);
const CACHE_CAPACITY: usize = 2_000;

type Offers = Arc<[ScrapedOffer]>;
/// A store's answer to a normalised query, with when it was fetched.
type Cache = HashMap<(&'static str, String), (Instant, Offers)>;

#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("no answer within {} seconds", TIMEOUT.as_secs())]
    Timeout,
    #[error("query cannot be sent to this store")]
    Query,
}

/// How one store answered a search, reported to the user.
#[derive(Debug, Serialize)]
pub struct SourceReport {
    pub store_id: &'static str,
    pub ok: bool,
    pub elapsed_ms: u128,
}

pub struct LiveSearch {
    sources: Vec<Source>,
    cache: Mutex<Cache>,
}

struct Source {
    store: Box<dyn Store>,
    fetcher: Fetcher,
}

impl LiveSearch {
    pub fn new(stores: Vec<Box<dyn Store>>) -> reqwest::Result<Self> {
        let source = |store| Ok(Source { store, fetcher: Fetcher::new(PAUSE)? });
        Ok(Self { sources: stores.into_iter().map(source).collect::<reqwest::Result<_>>()?, cache: Mutex::default() })
    }

    pub fn stores(&self) -> impl Iterator<Item = &dyn Store> {
        self.sources.iter().map(|source| source.store.as_ref())
    }

    /// Searches the stores `wanted` accepts, all at once. `query` is the user's text;
    /// `key` is its normalised form, so differently typed but equal searches share
    /// cache entries.
    pub async fn search(&self, wanted: impl Fn(&str) -> bool, query: &str, key: &str) -> Vec<(SourceReport, Offers)> {
        let sources = self.sources.iter().filter(|source| wanted(source.store.id()));
        join_all(sources.map(|source| async move {
            let started = Instant::now();
            let result = self.cached(source, query, key).await;
            if let Err(error) = &result {
                tracing::warn!(store = source.store.id(), query, "search failed: {error}");
            }
            let report = SourceReport {
                store_id: source.store.id(),
                ok: result.is_ok(),
                elapsed_ms: started.elapsed().as_millis(),
            };
            (report, result.unwrap_or_default())
        }))
        .await
    }

    /// Runs a store's test query, bypassing the cache; used by the health check.
    pub async fn probe(&self, store_id: &str) -> Result<usize, LiveError> {
        let source = self.sources.iter().find(|source| source.store.id() == store_id).ok_or(LiveError::Query)?;
        Ok(fetch(source, source.store.probe()).await?.len())
    }

    async fn cached(&self, source: &Source, query: &str, key: &str) -> Result<Offers, LiveError> {
        let cache_key = (source.store.id(), key.to_owned());
        if let Some((stored, offers)) = self.lock().get(&cache_key)
            && stored.elapsed() < CACHE_TTL
        {
            return Ok(offers.clone());
        }
        let offers = fetch(source, query).await?;
        let mut cache = self.lock();
        if cache.len() >= CACHE_CAPACITY {
            cache.retain(|_, (stored, _)| stored.elapsed() < CACHE_TTL);
        }
        if cache.len() >= CACHE_CAPACITY {
            // Still full of fresh answers: start over rather than grow without bound.
            cache.clear();
        }
        cache.insert(cache_key, (Instant::now(), offers.clone()));
        Ok(offers)
    }

    fn lock(&self) -> MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

async fn fetch(source: &Source, query: &str) -> Result<Offers, LiveError> {
    let url = source.store.search_url(query).ok_or(LiveError::Query)?;
    let body = tokio::time::timeout(TIMEOUT, source.fetcher.get(&url)).await.map_err(|_| LiveError::Timeout)??;
    Ok(source.store.parse_results(&body, &url).into())
}
