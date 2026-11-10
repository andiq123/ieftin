//! The generic adapter: walks a store's sitemaps to its product pages and reads the
//! schema.org `Product` data (JSON-LD, or Open Graph product tags) that shops publish
//! for search engines. Most stores need nothing more than a registry entry.

use scraper::Html;
use serde_json::Value;
use url::Url;

use super::{
    Store, StoreInfo,
    parse::{self, selector},
};
use crate::domain::{Page, ScrapedOffer};

/// Filters are case-sensitive substrings of the URL; an empty list accepts everything.
pub struct Structured {
    id: &'static str,
    name: &'static str,
    home: &'static str,
    sitemap: &'static str,
    /// Sub-sitemaps worth following (those listing products).
    sitemaps: &'static [&'static str],
    /// Sitemap entries that are product pages.
    pages: &'static [&'static str],
    /// URLs to ignore, typically the Russian duplicate of every page.
    skip: &'static [&'static str],
}

impl Structured {
    /// A store whose sitemap lists product pages directly; narrow it with the filters.
    pub const fn new(id: &'static str, name: &'static str, home: &'static str, sitemap: &'static str) -> Self {
        Self { id, name, home, sitemap, sitemaps: &[], pages: &[], skip: &[] }
    }

    pub const fn sitemaps(mut self, patterns: &'static [&'static str]) -> Self {
        self.sitemaps = patterns;
        self
    }

    pub const fn pages(mut self, patterns: &'static [&'static str]) -> Self {
        self.pages = patterns;
        self
    }

    pub const fn skip(mut self, patterns: &'static [&'static str]) -> Self {
        self.skip = patterns;
        self
    }
}

impl StoreInfo for Structured {
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

impl Store for Structured {
    fn entry(&self) -> &'static str {
        self.sitemap
    }

    fn parse(&self, body: &str, url: &Url) -> Page {
        if !parse::is_sitemap(body) {
            return Page { offers: offers(body, url), links: Vec::new() };
        }
        let wanted = if body.contains("<sitemapindex") { self.sitemaps } else { self.pages };
        let matches = |patterns: &[&str], link: &Url| patterns.iter().any(|part| link.as_str().contains(part));
        let links = parse::sitemap_locs(body)
            .filter(|link| wanted.is_empty() || matches(wanted, link))
            .filter(|link| !matches(self.skip, link))
            .collect();
        Page { offers: Vec::new(), links }
    }
}

fn offers(html: &str, page: &Url) -> Vec<ScrapedOffer> {
    let document = Html::parse_document(html);
    let products: Vec<Value> = document
        .select(selector!(r#"script[type="application/ld+json"]"#))
        .filter_map(|script| serde_json::from_str(&script.text().collect::<String>()).ok())
        .flat_map(|data: Value| products_in(data))
        .collect();

    // A product without its own URL belongs to the page it is on, which is only
    // unambiguous when it is the page's single product.
    let page_url = (products.len() == 1).then_some(page);
    let offers: Vec<_> = products.iter().filter_map(|product| json_ld_offer(product, page, page_url)).collect();
    if offers.is_empty() { open_graph_offer(&document, page).into_iter().collect() } else { offers }
}

/// Every schema.org `Product` node, however deeply the document nests it.
fn products_in(data: Value) -> Vec<Value> {
    let is_product = |node: &Value| match &node["@type"] {
        Value::String(kind) => kind == "Product",
        Value::Array(kinds) => kinds.iter().any(|kind| kind == "Product"),
        _ => false,
    };
    match data {
        node if is_product(&node) => vec![node],
        Value::Array(items) => items.into_iter().flat_map(products_in).collect(),
        Value::Object(fields) => fields.into_iter().flat_map(|(_, value)| products_in(value)).collect(),
        _ => Vec::new(),
    }
}

fn json_ld_offer(product: &Value, page: &Url, page_url: Option<&Url>) -> Option<ScrapedOffer> {
    // `offers` is one offer or a list. Price ranges (AggregateOffer) are skipped:
    // they describe a group of products, not something one can buy at that price.
    let offer = match &product["offers"] {
        Value::Array(offers) => offers.first()?,
        offer => offer,
    };
    if offer["@type"] == "AggregateOffer" || !is_lei(offer["priceCurrency"].as_str()) {
        return None;
    }
    let url = match product["url"].as_str().or(offer["url"].as_str()) {
        Some(url) => page.join(url).ok()?,
        None => page_url?.clone(),
    };
    let availability = offer["availability"].as_str().unwrap_or_default();

    Some(ScrapedOffer {
        external_id: external_id(&url),
        title: parse::plain_text(product["name"].as_str()?)?,
        image_url: first_string(&product["image"]).and_then(|image| page.join(image).ok()),
        category: product["category"].as_str().and_then(parse::plain_text),
        price: parse::price(&scalar(&offer["price"])?)?,
        old_price: None,
        in_stock: if availability.ends_with("InStock") {
            Some(true)
        } else if availability.is_empty() {
            None
        } else {
            Some(false)
        },
        url,
    })
}

/// Fallback for shops that only publish Open Graph product tags.
fn open_graph_offer(document: &Html, page: &Url) -> Option<ScrapedOffer> {
    let meta = |selector| document.select(selector).next().and_then(|tag| tag.attr("content"));
    if !is_lei(meta(selector!(r#"meta[property="product:price:currency"]"#))) {
        return None;
    }
    Some(ScrapedOffer {
        external_id: external_id(page),
        title: parse::plain_text(meta(selector!(r#"meta[property="og:title"]"#))?)?,
        url: page.clone(),
        image_url: meta(selector!(r#"meta[property="og:image"]"#)).and_then(|image| page.join(image).ok()),
        category: None,
        price: parse::price(meta(selector!(r#"meta[property="product:price:amount"]"#))?)?,
        old_price: None,
        in_stock: None,
    })
}

/// Prices are compared in lei; an unstated currency is assumed to be lei.
fn is_lei(currency: Option<&str>) -> bool {
    currency.is_none_or(|currency| currency.eq_ignore_ascii_case("MDL"))
}

fn external_id(url: &Url) -> String {
    url[url::Position::BeforePath..url::Position::AfterQuery].to_owned()
}

/// A JSON number or string as text.
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

/// `image` may be a URL, a list of URLs, or an `ImageObject`.
fn first_string(value: &Value) -> Option<&str> {
    match value {
        Value::String(text) => Some(text),
        Value::Array(items) => items.first().and_then(first_string),
        Value::Object(_) => value["url"].as_str(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STORE: Structured = Structured::new("shop", "Shop", "https://shop.md", "https://shop.md/sitemap.xml")
        .sitemaps(&["products"])
        .pages(&["/product/"])
        .skip(&["/ru/"]);

    fn parse(body: &str, url: &str) -> Page {
        STORE.parse(body, &Url::parse(url).unwrap())
    }

    #[test]
    fn follows_only_wanted_sitemap_entries() {
        let index = "<?xml version=\"1.0\"?><sitemapindex><loc>https://shop.md/products-1.xml</loc>\
                     <loc>https://shop.md/blog.xml</loc></sitemapindex>";
        assert_eq!(parse(index, STORE.sitemap).links.len(), 1);

        let urls = "<?xml version=\"1.0\"?><urlset><loc>https://shop.md/product/a</loc>\
                    <loc>https://shop.md/ru/product/a</loc><loc>https://shop.md/about</loc></urlset>";
        let links = parse(urls, "https://shop.md/products-1.xml").links;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].path(), "/product/a");
    }

    #[test]
    fn reads_json_ld_product() {
        let html = r#"<script type="application/ld+json">{"@graph":[{"@type":"Product",
            "name":"Apple iPhone 15 Pro &amp; Case","image":["/img/a.jpg"],
            "offers":{"@type":"Offer","price":"17499.00","priceCurrency":"MDL",
                      "availability":"https://schema.org/InStock"}}]}</script>"#;
        let page = parse(html, "https://shop.md/product/iphone?color=blue");
        let [offer] = page.offers.as_slice() else { panic!("expected one offer") };
        assert_eq!(offer.external_id, "/product/iphone?color=blue");
        assert_eq!(offer.title, "Apple iPhone 15 Pro & Case");
        assert_eq!((offer.price, offer.in_stock), (1_749_900, Some(true)));
        assert_eq!(offer.image_url.as_ref().unwrap().as_str(), "https://shop.md/img/a.jpg");
    }

    #[test]
    fn ignores_price_ranges_and_foreign_currencies() {
        let range = r#"<script type="application/ld+json">{"@type":"Product","name":"Phones",
            "offers":{"@type":"AggregateOffer","lowPrice":100,"priceCurrency":"MDL"}}</script>"#;
        let euros = r#"<script type="application/ld+json">{"@type":"Product","name":"Phone",
            "offers":{"@type":"Offer","price":100,"priceCurrency":"EUR"}}</script>"#;
        assert!(parse(range, "https://shop.md/product/a").offers.is_empty());
        assert!(parse(euros, "https://shop.md/product/a").offers.is_empty());
    }

    #[test]
    fn falls_back_to_open_graph() {
        let html = r#"<meta property="og:title" content="LG TV 55"><meta property="product:price:amount" content="8990">
            <meta property="product:price:currency" content="MDL">"#;
        let page = parse(html, "https://shop.md/product/tv");
        assert_eq!(page.offers[0].title, "LG TV 55");
        assert_eq!(page.offers[0].price, 899_000);
    }
}
