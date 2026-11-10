//! Stores searched live: instead of crawling the catalogue, one request to the store's
//! own search page is made when a user searches.

use scraper::{ElementRef, Html, Selector};
use url::Url;

use super::{StoreInfo, parse};
use crate::domain::ScrapedOffer;

/// Like [`super::Store`], implementations only build a URL and read a document.
pub trait LiveStore: StoreInfo {
    /// A query that should always return results, used by the daily health check.
    fn probe(&self) -> &'static str;
    /// The store's search results page for `query`.
    fn search_url(&self, query: &str) -> Option<Url>;
    /// Extracts the offers from a search results page.
    fn parse_results(&self, body: &str, url: &Url) -> Vec<ScrapedOffer>;
}

/// Builds a search URL from a template whose `{}` stands for the query.
pub fn search_url(template: &str, query: &str) -> Option<Url> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    Url::parse(&template.replace("{}", &encoded)).ok()
}

/// A store whose search results are plain HTML cards, described entirely by CSS
/// selectors. Adding such a store is a registry entry, not code.
pub struct CssSearch {
    pub id: &'static str,
    pub name: &'static str,
    pub home: &'static str,
    /// Search URL with `{}` for the query.
    pub search: &'static str,
    pub probe: &'static str,
    /// One product card. The remaining selectors are relative to it.
    pub card: &'static str,
    /// The link to the product page; its text is the title unless `title` is set.
    pub link: &'static str,
    pub title: Option<&'static str>,
    pub price: &'static str,
    pub old_price: Option<&'static str>,
    /// Image element and the attribute holding its URL.
    pub image: Option<(&'static str, &'static str)>,
}

impl StoreInfo for CssSearch {
    fn id(&self) -> &'static str {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn home(&self) -> &'static str {
        self.home
    }
}

impl LiveStore for CssSearch {
    fn probe(&self) -> &'static str {
        self.probe
    }

    fn search_url(&self, query: &str) -> Option<Url> {
        search_url(self.search, query)
    }

    fn parse_results(&self, body: &str, url: &Url) -> Vec<ScrapedOffer> {
        let Ok(card) = Selector::parse(self.card) else { return Vec::new() };
        Html::parse_document(body).select(&card).filter_map(|card| self.offer(card, url)).collect()
    }
}

impl CssSearch {
    fn offer(&self, card: ElementRef, page: &Url) -> Option<ScrapedOffer> {
        let first = |css: &str| Selector::parse(css).ok().and_then(|selector| card.select(&selector).next());
        let price_of = |css: &str| first(css).map(parse::text).as_deref().and_then(parse::price);
        let link = first(self.link)?;
        let url = parse::link(link, "href", page)?;

        Some(ScrapedOffer {
            external_id: url.path().to_owned(),
            title: Some(parse::text(self.title.and_then(first).unwrap_or(link))).filter(|title| !title.is_empty())?,
            image_url: self.image.and_then(|(css, attribute)| parse::link(first(css)?, attribute, page)),
            category: None,
            price: price_of(self.price)?,
            old_price: self.old_price.and_then(price_of),
            in_stock: None,
            url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOP: CssSearch = CssSearch {
        id: "shop",
        name: "Shop",
        home: "https://shop.md",
        search: "https://shop.md/search?q={}",
        probe: "samsung",
        card: ".card",
        link: "a.name",
        title: None,
        price: ".price",
        old_price: Some(".old"),
        image: Some(("img", "src")),
    };

    #[test]
    fn builds_an_encoded_search_url() {
        let url = SHOP.search_url("iphone 15 & pro").unwrap();
        assert_eq!(url.as_str(), "https://shop.md/search?q=iphone+15+%26+pro");
    }

    #[test]
    fn reads_cards() {
        let html = r#"<div class="card"><img src="/a.jpg"><a class="name" href="/p/1"> Galaxy  S24 </a>
            <span class="old">18.999 MDL</span><span class="price">17 499 lei</span></div>
            <div class="card"><a class="name" href="/p/2">No price</a></div>"#;
        let offers = SHOP.parse_results(html, &Url::parse("https://shop.md/search?q=galaxy").unwrap());
        let [offer] = offers.as_slice() else { panic!("expected one offer") };
        assert_eq!((offer.title.as_str(), offer.external_id.as_str()), ("Galaxy S24", "/p/1"));
        assert_eq!((offer.price, offer.old_price), (1_749_900, Some(1_899_900)));
        assert_eq!(offer.image_url.as_ref().unwrap().as_str(), "https://shop.md/a.jpg");
    }
}
