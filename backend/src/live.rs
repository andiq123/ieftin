//! Searching: asks each store's own search page for the query and reads the results.
//!
//! Stores answer at very different speeds, so a search does not wait for them all: it
//! answers with whatever arrived in the first moment, and the rest keep going in the
//! background. Their answers land in a short-lived cache, where a repeat of the search
//! picks them up, so a page that keeps asking sees results arrive store by store. The
//! same cache makes paging cost the stores nothing.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use futures::{StreamExt, stream::FuturesUnordered};
use serde::Serialize;
use tokio::time::Instant;

use crate::{
    domain::ScrapedOffer,
    fetch::{FetchError, Fetcher},
    stores::Store,
};

/// How long a search gathers answers before showing what it has, once it has any.
const FIRST_LOOK: Duration = Duration::from_millis(1200);
/// The longest a search waits for a first answer before answering with none.
const SEARCH_LIMIT: Duration = Duration::from_secs(8);
/// How long a store may take in the background, or in a health test, where nobody is
/// waiting: what matters there is whether it answers at all.
const STORE_LIMIT: Duration = Duration::from_secs(25);
/// Minimum time between two requests to the same store.
const PAUSE: Duration = Duration::from_millis(300);
/// How long a store's answer to a query is reused. Prices rarely move within minutes.
const CACHE_TTL: Duration = Duration::from_secs(600);
const CACHE_CAPACITY: usize = 2_000;

type Offers = Arc<[ScrapedOffer]>;
/// A store's answer to one search and when it arrived; `None` while it is still out.
type Answer = Option<(Result<Offers, LiveError>, Instant)>;
/// A store and a normalised query.
type Key = (&'static str, String);

#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("no answer in time")]
    Timeout,
    #[error("query cannot be sent to this store")]
    Query,
}

/// How one store answered a search, reported to the user.
#[derive(Debug, Serialize)]
pub struct SourceReport {
    pub store_id: &'static str,
    pub ok: bool,
    /// Still being read in the background: a repeat of the search will have it.
    pub pending: bool,
    pub elapsed_ms: u128,
}

pub struct LiveSearch {
    sources: Vec<Arc<Source>>,
    /// Answers by store and query, with when each was fetched.
    cache: Mutex<HashMap<Key, (Instant, Offers)>>,
    /// Requests under way, so a repeated search does not ask a slow store twice.
    running: Mutex<HashSet<Key>>,
}

struct Source {
    store: Box<dyn Store>,
    fetcher: Fetcher,
}

impl LiveSearch {
    pub fn new(stores: Vec<Box<dyn Store>>) -> reqwest::Result<Self> {
        let source = |store| Ok(Arc::new(Source { store, fetcher: Fetcher::new(PAUSE)? }));
        Ok(Self {
            sources: stores.into_iter().map(source).collect::<reqwest::Result<_>>()?,
            cache: Mutex::default(),
            running: Mutex::default(),
        })
    }

    pub fn stores(&self) -> impl Iterator<Item = &dyn Store> {
        self.sources.iter().map(|source| source.store.as_ref())
    }

    /// Searches the stores `wanted` accepts, all at once. `query` is the user's text;
    /// `key` is its normalised form, so differently typed but equal searches share
    /// cache entries.
    pub async fn search(
        self: &Arc<Self>,
        wanted: impl Fn(&str) -> bool,
        query: &str,
        key: &str,
    ) -> Vec<(SourceReport, Offers)> {
        let started = Instant::now();
        let sources: Vec<_> = self.sources.iter().filter(|source| wanted(source.store.id())).collect();
        let mut answers: Vec<Answer> = Vec::new();
        let mut waiting = FuturesUnordered::new();

        for (index, source) in sources.iter().enumerate() {
            let id = (source.store.id(), key.to_owned());
            if let Some(offers) = self.kept(&id) {
                answers.push(Some((Ok(offers), started)));
                continue;
            }
            answers.push(None);
            // Already being asked by an earlier search: its answer will be in the cache.
            if !lock(&self.running).insert(id.clone()) {
                continue;
            }
            let (this, source, query) = (self.clone(), (*source).clone(), query.to_owned());
            let request = tokio::spawn(async move {
                let result = fetch(&source, &query, STORE_LIMIT).await;
                this.finish(id, &result);
                result
            });
            // Dropping the handle later does not stop the request.
            waiting.push(async move { (index, request.await.unwrap_or(Err(LiveError::Timeout))) });
        }

        // Wait for everyone only briefly; with nothing at all yet, wait for a first answer.
        let mut deadline =
            if answers.iter().any(Option::is_some) { started + FIRST_LOOK } else { started + SEARCH_LIMIT };
        while let Ok(Some((index, result))) = tokio::time::timeout_at(deadline, waiting.next()).await {
            if let Err(error) = &result {
                tracing::warn!(store = sources[index].store.id(), query, "search failed: {error}");
            }
            answers[index] = Some((result, Instant::now()));
            deadline = deadline.min((started + FIRST_LOOK).max(Instant::now()));
        }

        let report = |(source, answer): (&&Arc<Source>, Answer)| {
            let (result, at) = answer.map_or((None, Instant::now()), |(result, at)| (Some(result), at));
            let report = SourceReport {
                store_id: source.store.id(),
                ok: matches!(result, Some(Ok(_))),
                pending: result.is_none(),
                elapsed_ms: at.duration_since(started).as_millis(),
            };
            (report, result.and_then(Result::ok).unwrap_or_default())
        };
        sources.iter().zip(answers).map(report).collect()
    }

    /// Whether searching `key` would send requests to stores, rather than be answered
    /// from what is cached or already under way.
    pub fn is_new(&self, wanted: impl Fn(&str) -> bool, key: &str) -> bool {
        let running = lock(&self.running);
        self.sources.iter().filter(|source| wanted(source.store.id())).any(|source| {
            let id = (source.store.id(), key.to_owned());
            !running.contains(&id) && self.kept(&id).is_none()
        })
    }

    /// Runs a store's test query, bypassing the cache; used by the health check.
    pub async fn probe(&self, store_id: &str) -> Result<usize, LiveError> {
        let source = self.sources.iter().find(|source| source.store.id() == store_id).ok_or(LiveError::Query)?;
        Ok(fetch(source, source.store.probe(), STORE_LIMIT).await?.len())
    }

    /// A store's answer to a query, while it is fresh.
    fn kept(&self, id: &Key) -> Option<Offers> {
        let cache = lock(&self.cache);
        cache.get(id).filter(|(stored, _)| stored.elapsed() < CACHE_TTL).map(|(_, offers)| offers.clone())
    }

    /// Records that a request ended, keeping a good answer for later searches.
    fn finish(&self, id: Key, result: &Result<Offers, LiveError>) {
        lock(&self.running).remove(&id);
        let Ok(offers) = result else { return };
        let mut cache = lock(&self.cache);
        if cache.len() >= CACHE_CAPACITY {
            cache.retain(|_, (stored, _)| stored.elapsed() < CACHE_TTL);
        }
        if cache.len() >= CACHE_CAPACITY {
            // Still full of fresh answers: start over rather than grow without bound.
            cache.clear();
        }
        cache.insert(id, (Instant::now(), offers.clone()));
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn fetch(source: &Source, query: &str, limit: Duration) -> Result<Offers, LiveError> {
    let url = source.store.search_url(query).ok_or(LiveError::Query)?;
    // The store's rules are read once, outside the time allowed for its answer.
    source.fetcher.prepare(&url).await;
    let attempts = async {
        match source.fetcher.get(&url).await {
            // A connection that could not be opened is worth one more try.
            Err(FetchError::Network(error)) if error.is_connect() => source.fetcher.get(&url).await,
            answer => answer,
        }
    };
    let body = tokio::time::timeout(limit, attempts).await.map_err(|_| LiveError::Timeout)??;
    Ok(source.store.parse_results(&body, &url).into())
}
