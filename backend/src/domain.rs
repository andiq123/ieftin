use serde::Serialize;
use url::Url;

/// Money is kept as integer bani (1 leu = 100 bani) end to end.
pub type Bani = i64;

/// One product as read from a store's search results.
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

/// An offer as shown to the user.
#[derive(Debug, Clone, Serialize)]
pub struct Offer {
    pub store_id: String,
    /// With `store_id`, identifies the offer, e.g. among a user's saved ones.
    pub external_id: String,
    pub title: String,
    pub url: String,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub price_bani: Bani,
    pub old_price_bani: Option<Bani>,
    pub in_stock: Option<bool>,
}

impl Offer {
    pub fn new(store_id: &str, scraped: &ScrapedOffer) -> Self {
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
        }
    }
}
