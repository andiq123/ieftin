use scraper::{ElementRef, Html};
use serde::Deserialize;
use url::Url;

use super::{
    Store,
    parse::{self, selector},
    search_url,
};
use crate::domain::{Bani, ScrapedOffer};

/// Darwin: its search results are product cards that each carry an analytics payload.
pub struct Darwin;

impl Store for Darwin {
    fn id(&self) -> &'static str {
        "darwin"
    }

    fn name(&self) -> &'static str {
        "Darwin"
    }

    fn home(&self) -> &'static str {
        "https://darwin.md"
    }

    fn probe(&self) -> &'static str {
        "samsung"
    }

    fn search_url(&self, query: &str) -> Option<Url> {
        search_url("https://darwin.md/cautare?keywords={}", query)
    }

    fn parse_results(&self, body: &str, url: &Url) -> Vec<ScrapedOffer> {
        offers(&Html::parse_document(body), url)
    }
}

fn offers(document: &Html, page: &Url) -> Vec<ScrapedOffer> {
    document.select(selector!("div.product-card")).filter_map(|card| offer(card, page)).collect()
}

/// The analytics payload each product card carries. Unlike the visible markup it
/// does not change with the site's design, so prices are read from it.
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
    // The payload sits on the product link, or on the card itself.
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

    const RESULTS: &str = r#"
        <div class="product-card product-item">
          <div class="product-img"><img data-src="/media/1/thumb.png"></div>
          <a class="stretched-link product-link" href="https://darwin.md/iphone.html"
             data-ga4="{&quot;ecommerce&quot;:{&quot;value&quot;:30999,&quot;items&quot;:[{&quot;item_id&quot;:&quot;00-1&quot;,&quot;discount&quot;:1900,&quot;item_category&quot;:&quot;Smartphone&quot;}]}}">
            <div class="title-product"> Apple iPhone 15 Pro
              256 GB </div>
          </a>
        </div>"#;

    #[test]
    fn reads_results() {
        let url = Darwin.search_url("iphone 15").unwrap();
        assert_eq!(url.as_str(), "https://darwin.md/cautare?keywords=iphone+15");
        let offers = Darwin.parse_results(RESULTS, &url);
        let [offer] = offers.as_slice() else { panic!("expected one offer") };
        assert_eq!(offer.external_id, "00-1");
        assert_eq!(offer.title, "Apple iPhone 15 Pro 256 GB");
        assert_eq!(offer.category.as_deref(), Some("Smartphone"));
        assert_eq!((offer.price, offer.old_price), (3_099_900, Some(3_289_900)));
        assert_eq!(offer.image_url.as_ref().unwrap().as_str(), "https://darwin.md/media/1/thumb.png");
    }
}
