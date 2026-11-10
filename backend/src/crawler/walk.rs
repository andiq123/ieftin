use std::collections::HashSet;

use url::Url;

use super::fetcher::{FetchError, Fetcher};
use crate::{domain::Page, stores::Store};

/// Depth-first traversal of a store, from its entry document through every link the
/// adapter returns. Depth-first reaches product data within a few requests, which is
/// what lets a health check and a full crawl share this code.
pub struct Walk<'a> {
    store: &'a dyn Store,
    fetcher: &'a Fetcher,
    /// Sitemaps are read before pages, so the amount of work is known early.
    sitemaps: Vec<Url>,
    pending: Vec<Url>,
    seen: HashSet<Url>,
    done: usize,
    sampling: bool,
}

/// Whether a link leads to another sitemap rather than a page, judged by its URL.
fn is_sitemap(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    path.contains("sitemap") || path.ends_with(".xml") || path.ends_with(".xml.gz")
}

impl<'a> Walk<'a> {
    pub fn new(store: &'a dyn Store, fetcher: &'a Fetcher) -> Result<Self, url::ParseError> {
        let entry = Url::parse(store.entry())?;
        Ok(Self {
            store,
            fetcher,
            seen: HashSet::from([entry.clone()]),
            sitemaps: Vec::new(),
            pending: vec![entry],
            done: 0,
            sampling: false,
        })
    }

    /// Visits pending documents in a scattered order instead of in sequence. A health
    /// check uses this so a sitemap that opens with a run of non-product pages
    /// cannot exhaust its budget.
    pub fn sampling(mut self) -> Self {
        self.sampling = true;
        self
    }

    /// Documents read so far.
    pub fn done(&self) -> usize {
        self.done
    }

    /// Documents discovered and not yet read.
    pub fn pending(&self) -> usize {
        self.sitemaps.len() + self.pending.len()
    }

    /// Fetches and parses the next document; `None` once the store is exhausted.
    pub async fn next(&mut self) -> Option<Result<Page, FetchError>> {
        if self.sampling && !self.pending.is_empty() {
            // Deterministic scatter (Fibonacci hashing on the visit count).
            let pick = self.seen.len().wrapping_mul(0x9E37_79B9) % self.pending.len();
            let last = self.pending.len() - 1;
            self.pending.swap(pick, last);
        }
        let url = self.sitemaps.pop().or_else(|| self.pending.pop())?;
        self.done += 1;
        let body = match self.fetcher.get(&url).await {
            Ok(body) => body,
            Err(error) => return Some(Err(error)),
        };
        let page = self.store.parse(&body, &url);
        // Reversed, so the first link is the next one popped.
        for link in page.links.iter().rev() {
            if self.seen.insert(link.clone()) {
                let queue = if is_sitemap(link) && !self.sampling { &mut self.sitemaps } else { &mut self.pending };
                queue.push(link.clone());
            }
        }
        Some(Ok(page))
    }
}
