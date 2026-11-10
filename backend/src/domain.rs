use chrono::{DateTime, Utc};
use serde::Serialize;
use url::Url;

/// Money is kept as integer bani (1 leu = 100 bani) end to end.
pub type Bani = i64;

/// One product as scraped from a store page.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrapedOffer {
    /// The store's own stable product id.
    pub external_id: String,
    pub title: String,
    pub url: Url,
    pub image_url: Option<Url>,
    pub category: Option<String>,
    pub price: Bani,
    /// Crossed-out price, when the store shows a discount.
    pub old_price: Option<Bani>,
    /// `None` when the page does not say.
    pub in_stock: Option<bool>,
}

/// What one fetched document contributes to a crawl.
#[derive(Debug, Default)]
pub struct Page {
    pub offers: Vec<ScrapedOffer>,
    /// Further documents to fetch: sub-sitemaps, categories, next pages, products.
    pub links: Vec<Url>,
}

/// Outcome of a store health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Health {
    /// Prices can be read.
    Ok,
    /// The store refuses automated access (HTTP 401/403/429 or robots.txt).
    Blocked,
    /// The store could not be reached.
    Unreachable,
    /// Pages load, but no readable price was found.
    NoData,
}

/// An offer as shown to the user, whether it came from the index or a live search.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Offer {
    pub store_id: String,
    /// With `store_id`, names the offer's price history.
    pub external_id: String,
    pub title: String,
    pub url: String,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub price_bani: Bani,
    pub old_price_bani: Option<Bani>,
    pub in_stock: Option<bool>,
    /// Read from the store just now, rather than from the daily index.
    #[sqlx(default)]
    pub live: bool,
}

impl Offer {
    pub fn live(store_id: &str, scraped: &ScrapedOffer) -> Self {
        Self {
            store_id: store_id.to_owned(),
            external_id: scraped.external_id.clone(),
            title: scraped.title.clone(),
            url: scraped.url.to_string(),
            image_url: scraped.image_url.as_ref().map(ToString::to_string),
            category: scraped.category.clone(),
            price_bani: scraped.price,
            old_price_bani: scraped.old_price,
            in_stock: scraped.in_stock,
            live: true,
        }
    }
}

/// A price an offer changed to, and when that was first seen.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PricePoint {
    pub price_bani: Bani,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StoreOverview {
    pub id: String,
    pub name: String,
    pub url: String,
    /// `live` (searched at query time) or `indexed` (crawled daily).
    pub mode: String,
    pub offers: i64,
    pub health_status: Option<Health>,
    pub health_detail: Option<String>,
    pub health_checked_at: Option<DateTime<Utc>>,
    pub last_crawl_started_at: Option<DateTime<Utc>>,
    pub last_crawl_finished_at: Option<DateTime<Utc>>,
}
