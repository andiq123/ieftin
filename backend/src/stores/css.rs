//! Stores whose search results are plain HTML cards, read with CSS selectors.

use scraper::{ElementRef, Html, Selector};
use url::Url;

use super::{Store, parse, search_url};
use crate::domain::ScrapedOffer;

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
    /// Where the title is instead; several matching elements are read in order.
    pub title: Option<&'static str>,
    pub price: &'static str,
    pub old_price: Option<&'static str>,
    /// Image element and the attribute holding its URL.
    pub image: Option<(&'static str, &'static str)>,
}

impl Store for CssSearch {
    fn id(&self) -> &'static str {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn home(&self) -> &'static str {
        self.home
    }

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
    /// The text of every element the title selector matches, joined in page order.
    fn title_of(&self, card: ElementRef) -> Option<String> {
        let selector = Selector::parse(self.title?).ok()?;
        let parts: Vec<String> = card.select(&selector).map(parse::text).filter(|part| !part.is_empty()).collect();
        (!parts.is_empty()).then(|| parts.join(" "))
    }

    fn offer(&self, card: ElementRef, page: &Url) -> Option<ScrapedOffer> {
        let first = |css: &str| Selector::parse(css).ok().and_then(|selector| card.select(&selector).next());
        let price_of = |css: &str| first(css).and_then(parse::price_in);
        let link = first(self.link)?;
        let url = parse::link(link, "href", page)?;

        Some(ScrapedOffer {
            external_id: url.path().to_owned(),
            title: Some(self.title_of(card).unwrap_or_else(|| parse::text(link))).filter(|title| !title.is_empty())?,
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
    fn joins_a_title_kept_in_several_elements() {
        let shop = CssSearch { title: Some(".model, .variant"), ..SHOP };
        let html = r#"<div class="card"><a class="name" href="/p/1"><b class="model">Apple iPhone 15</b></a>
            <span class="variant">6 GB / 128 GB / Blue</span><span class="price">14 999 lei</span></div>"#;
        let offers = shop.parse_results(html, &Url::parse("https://shop.md/search?q=iphone").unwrap());
        assert_eq!(offers[0].title, "Apple iPhone 15 6 GB / 128 GB / Blue");
    }

    #[test]
    fn builds_an_encoded_search_url() {
        let url = SHOP.search_url("iphone 15 & pro").unwrap();
        assert_eq!(url.as_str(), "https://shop.md/search?q=iphone+15+%26+pro");
    }

    #[test]
    fn reads_cards() {
        let html = r#"<div class="card"><img src="/a.jpg"><a class="name" href="/p/1"> Galaxy  S24 </a>
            <span class="old">18.999 MDL</span><span class="price">17 499 lei</span></div>
            <div class="card"><a class="name" href="/p/2">No price</a></div>
            <div class="card"><a class="name" href="/p/3">Discounted</a>
              <span class="price"><span class="old">899<i>LEI</i></span> 555 </span></div>"#;
        let offers = SHOP.parse_results(html, &Url::parse("https://shop.md/search?q=galaxy").unwrap());
        let [offer, discounted] = offers.as_slice() else { panic!("expected two offers") };
        assert_eq!((offer.title.as_str(), offer.external_id.as_str()), ("Galaxy S24", "/p/1"));
        // The old price nested inside the price element is not part of the price.
        assert_eq!((discounted.price, discounted.old_price), (55_500, Some(89_900)));
        assert_eq!((offer.price, offer.old_price), (1_749_900, Some(1_899_900)));
        assert_eq!(offer.image_url.as_ref().unwrap().as_str(), "https://shop.md/a.jpg");
    }
}
