//! Store adapters. A store is searched through its own search page at the moment a
//! user searches, so every price shown was read just now.
//!
//! Adapters only build a URL and read a document; fetching, health checks and
//! ranking are shared. Most stores are a single entry in [`registry`].

mod css;
mod darwin;
mod parse;
mod registry;

use url::Url;

pub use self::registry::all;
use crate::domain::ScrapedOffer;

/// One store. Implementations do no I/O, which keeps them trivial to test against
/// saved documents.
pub trait Store: Send + Sync {
    /// Stable identifier.
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn home(&self) -> &'static str;
    /// A query that should always return results, used by the health check.
    fn probe(&self) -> &'static str;
    /// The store's search results page for `query`.
    fn search_url(&self, query: &str) -> Option<Url>;
    /// Extracts the offers from a search results page.
    fn parse_results(&self, body: &str, url: &Url) -> Vec<ScrapedOffer>;
}

/// Builds a search URL from a template whose `{}` stands for the query.
fn search_url(template: &str, query: &str) -> Option<Url> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    Url::parse(&template.replace("{}", &encoded)).ok()
}
