use scraper::{ElementRef, Html};
use serde::Deserialize;
use url::Url;

use super::{
    Store, StoreInfo,
    live::{self, LiveStore},
    parse::{self, selector},
};
use crate::domain::{Bani, Page, ScrapedOffer};

/// The e-commerce platform behind Darwin, Enter and iStore: a category sitemap and
/// paginated category listings with identical markup.
pub struct DarwinPlatform {
    id: &'static str,
    name: &'static str,
    home: &'static str,
    /// The categories sitemap.
    categories: &'static str,
}

impl DarwinPlatform {
    pub const fn new(id: &'static str, name: &'static str, home: &'static str, categories: &'static str) -> Self {
        Self { id, name, home, categories }
    }
}

impl StoreInfo for DarwinPlatform {
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

impl Store for DarwinPlatform {
    fn entry(&self) -> &'static str {
        self.categories
    }

    fn parse(&self, body: &str, url: &Url) -> Page {
        if parse::is_sitemap(body) {
            // The sitemap lists every category twice; keep the Romanian one.
            let romanian = parse::sitemap_locs(body)
                .filter(|url| url.path_segments().is_some_and(|mut path| path.next() != Some("ru")));
            return Page { offers: Vec::new(), links: parse::leaves(romanian) };
        }
        let document = Html::parse_document(body);
        let next = page_number(url) + 1;
        Page {
            offers: offers(&document, url),
            links: document
                .select(selector!("a[href*='page=']"))
                .filter_map(|anchor| parse::link(anchor, "href", url))
                .find(|link| page_number(link) == next)
                .into_iter()
                .collect(),
        }
    }
}

/// Darwin itself allows automated use of its search page, so it is queried live.
/// The results use the same product cards as the category listings.
pub struct DarwinSearch;

impl StoreInfo for DarwinSearch {
    fn id(&self) -> &'static str {
        "darwin"
    }

    fn name(&self) -> &'static str {
        "Darwin"
    }

    fn home(&self) -> &'static str {
        "https://darwin.md"
    }
}

impl LiveStore for DarwinSearch {
    fn probe(&self) -> &'static str {
        "samsung"
    }

    fn search_url(&self, query: &str) -> Option<Url> {
        live::search_url("https://darwin.md/cautare?keywords={}", query)
    }

    fn parse_results(&self, body: &str, url: &Url) -> Vec<ScrapedOffer> {
        offers(&Html::parse_document(body), url)
    }
}

fn offers(document: &Html, page: &Url) -> Vec<ScrapedOffer> {
    document.select(selector!("div.product-card")).filter_map(|card| offer(card, page)).collect()
}

/// The `page` query parameter; the first page has none.
fn page_number(url: &Url) -> u32 {
    url.query_pairs().find(|(key, _)| key == "page").and_then(|(_, page)| page.parse().ok()).unwrap_or(1)
}

/// The analytics payload each product card carries. It is the same on every skin of
/// the platform, unlike the visible markup, so prices are read from it.
#[derive(Deserialize)]
struct Analytics {
    ecommerce: Ecommerce,
}

#[derive(Deserialize)]
struct Ecommerce {
    /// Current price in lei.
    value: f64,
    items: Vec<AnalyticsItem>,
}

#[derive(Deserialize)]
struct AnalyticsItem {
    item_id: String,
    item_category: Option<String>,
    /// Lei taken off the regular price.
    #[serde(default)]
    discount: f64,
}

fn offer(card: ElementRef, page: &Url) -> Option<ScrapedOffer> {
    // Darwin and iStore put the payload on the product link, Enter on the card itself.
    let payload = card
        .attr("data-gtm")
        .or_else(|| card.select(selector!("[data-ga4]")).next().and_then(|link| link.attr("data-ga4")))?;
    let Analytics { ecommerce } = serde_json::from_str(payload).ok()?;
    let item = ecommerce.items.into_iter().next()?;
    let to_bani = |lei: f64| (lei * 100.0).round() as Bani;
    let price = Some(to_bani(ecommerce.value)).filter(|price| *price > 0)?;

    Some(ScrapedOffer {
        external_id: item.item_id,
        title: parse::text_of(card, selector!(".title-product, .product-title"))?,
        url: card.select(selector!("a.stretched-link")).next().and_then(|anchor| parse::link(anchor, "href", page))?,
        image_url: card
            .select(selector!(".product-img img"))
            .next()
            .and_then(|image| parse::link(image, "data-src", page)),
        category: item.item_category,
        price,
        old_price: (item.discount > 0.0).then(|| price + to_bani(item.discount)),
        in_stock: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DARWIN: DarwinPlatform =
        DarwinPlatform::new("darwin", "Darwin", "https://darwin.md", "https://darwin.md/media/sitemaps/categories.xml");

    const LISTING: &str = r#"
        <div class="product-card product-item">
          <div class="product-img"><img data-src="/media/1/thumb.png"></div>
          <a class="stretched-link product-link" href="https://darwin.md/iphone.html"
             data-ga4="{&quot;ecommerce&quot;:{&quot;value&quot;:30999,&quot;items&quot;:[{&quot;item_id&quot;:&quot;00-1&quot;,&quot;discount&quot;:1900,&quot;item_category&quot;:&quot;Smartphone&quot;}]}}">
            <div class="title-product"> Apple iPhone 15 Pro
              256 GB </div>
          </a>
        </div>
        <a href="https://darwin.md/telefoane/smartphone?page=75">75</a>
        <a href="https://darwin.md/telefoane/smartphone?page=2">2</a>"#;

    #[test]
    fn parses_listing() {
        let url = Url::parse("https://darwin.md/telefoane/smartphone").unwrap();
        let page = DARWIN.parse(LISTING, &url);
        assert_eq!(page.links[0].query(), Some("page=2"));
        let [offer] = page.offers.as_slice() else { panic!("expected one offer") };
        assert_eq!(offer.external_id, "00-1");
        assert_eq!(offer.title, "Apple iPhone 15 Pro 256 GB");
        assert_eq!(offer.category.as_deref(), Some("Smartphone"));
        assert_eq!((offer.price, offer.old_price), (3_099_900, Some(3_289_900)));
        assert_eq!(offer.image_url.as_ref().unwrap().as_str(), "https://darwin.md/media/1/thumb.png");
    }

    #[test]
    fn follows_romanian_leaf_categories() {
        let sitemap = "<urlset><loc>https://darwin.md/telefoane</loc><loc>https://darwin.md/ru/telefoane</loc>\
                       <loc>https://darwin.md/telefoane/smartphone</loc></urlset>";
        let links = DARWIN.parse(sitemap, &Url::parse(DARWIN.categories).unwrap()).links;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].path(), "/telefoane/smartphone");
    }
}
