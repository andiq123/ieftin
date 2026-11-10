//! Store adapters. Prices are read in one of two ways:
//!
//! - **live** ([`LiveStore`]): the store's own search page is queried when a user
//!   searches. Preferred, wherever the store's robots.txt allows it.
//! - **indexed** ([`Store`]): the catalogue is crawled daily into the database, for
//!   stores that forbid automated use of their search.
//!
//! Adapters only describe URLs and how to read a document; fetching, health checks,
//! storage and search are shared. Most stores are a single entry in [`registry`].

mod atehno;
mod darwin;
mod live;
mod maximum;
mod parse;
mod registry;
mod structured;

use url::Url;

pub use self::{
    live::LiveStore,
    registry::{Registry, registry},
};
use crate::domain::Page;

/// What every store has, however its prices are read.
pub trait StoreInfo: Send + Sync {
    /// Stable identifier, used as the primary key.
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn home(&self) -> &'static str;
}

/// A store indexed by crawling. Implementations do no I/O, which keeps them trivial
/// to test against saved documents.
pub trait Store: StoreInfo {
    /// The document a crawl starts from, usually a sitemap.
    fn entry(&self) -> &'static str;
    /// Extracts offers and further links from one fetched document.
    fn parse(&self, body: &str, url: &Url) -> Page;
}
