use scraper::{ElementRef, Html};
use url::Url;

use super::{
    Store, StoreInfo,
    parse::{self, selector},
};
use crate::domain::{Page, ScrapedOffer};

pub struct Maximum;

/// First path segments that look like categories in the sitemap but are not.
const NOT_CATEGORIES: [&str; 8] =
    ["article", "articles", "akcii", "brand", "page", "promo", "promotii-si-oferte-speciale", "Apple"];

impl StoreInfo for Maximum {
    fn id(&self) -> &'static str {
        "maximum"
    }

    fn name(&self) -> &'static str {
        "Maximum"
    }

    fn home(&self) -> &'static str {
        "https://maximum.md"
    }
}

impl Store for Maximum {
    fn entry(&self) -> &'static str {
        "https://maximum.md/sitemap.1.xml.gz"
    }

    fn parse(&self, body: &str, url: &Url) -> Page {
        if parse::is_sitemap(body) {
            return Page { offers: Vec::new(), links: parse::sitemap_locs(body).filter(is_category).collect() };
        }
        let document = Html::parse_document(body);
        let category = parse::text_of(document.root_element(), selector!("h1"));
        Page {
            offers: document
                .select(selector!("div.js-content.product__item"))
                .filter_map(|card| offer(card, category.as_deref(), url))
                .collect(),
            links: document
                .select(selector!("a[title='Pagina următoare']"))
                .next()
                .and_then(|anchor| parse::link(anchor, "href", url))
                .into_iter()
                .collect(),
        }
    }
}

/// Product listings live at exactly `/ro/<department>/<group>/<category>/`.
fn is_category(url: &Url) -> bool {
    let segments: Vec<&str> = url.path().split('/').filter(|segment| !segment.is_empty()).collect();
    url.query().is_none() && matches!(segments[..], ["ro", department, _, _] if !NOT_CATEGORIES.contains(&department))
}

fn offer(card: ElementRef, category: Option<&str>, page: &Url) -> Option<ScrapedOffer> {
    let anchor = card.select(selector!(".product__item__title a")).next()?;
    let price_of = |selector| parse::text_of(card, selector).as_deref().and_then(parse::price);

    Some(ScrapedOffer {
        external_id: card.select(selector!("[data-product]")).next()?.attr("data-product")?.to_owned(),
        title: Some(parse::text(anchor)).filter(|title| !title.is_empty())?,
        url: parse::link(anchor, "href", page)?,
        image_url: card
            .select(selector!(".product__item__image img"))
            .next()
            .and_then(|image| parse::link(image, "data-src", page)),
        category: category.map(str::to_owned),
        price: price_of(selector!(".product__item__price-current > span:first-child"))?,
        old_price: price_of(selector!(".product__item__price-old > span:first-child")),
        // The "not in stock" notice is always rendered and hidden while the product is available.
        in_stock: Some(card.select(selector!(".text_price_flex.hidden")).next().is_some()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = r#"
        <h1> Smartphone-uri </h1>
        <div class="js-content product__item ">
          <div class="product__item__image"><a href="/ro/6744031/"><img data-src="https://i.example/p.jpg"></a></div>
          <div class="product__item__title"><a href="/ro/6744031/"> Smartphone Apple iPhone 15 128GB Black </a></div>
          <div class="product__item__price-old"> </div>
          <div class="product__item__price-current"><span> 14999 </span> <span>lei</span></div>
          <div class="text_price_flex hidden"></div>
          <button class="js-add-to-cart" data-product="6744031"></button>
        </div>
        <a title="Pagina următoare" href="/ro/a/b/c/2">mai departe</a>"#;

    #[test]
    fn parses_listing() {
        let url = Url::parse("https://maximum.md/ro/a/b/c/").unwrap();
        let page = Maximum.parse(LISTING, &url);
        assert_eq!(page.links[0].path(), "/ro/a/b/c/2");
        let [offer] = page.offers.as_slice() else { panic!("expected one offer") };
        assert_eq!(offer.external_id, "6744031");
        assert_eq!(offer.url.as_str(), "https://maximum.md/ro/6744031/");
        assert_eq!(offer.category.as_deref(), Some("Smartphone-uri"));
        assert_eq!((offer.price, offer.old_price, offer.in_stock), (1_499_900, None, Some(true)));
    }

    #[test]
    fn recognises_categories() {
        let index = "<urlset><loc>https://maximum.md/ro/televizoare/televizoare/televizoare/</loc>\
                     <loc>https://maximum.md/ru/televizoare/televizoare/televizoare/</loc>\
                     <loc>https://maximum.md/ro/article/a/b/</loc>\
                     <loc>https://maximum.md/ro/6744031/</loc>";
        let url = Url::parse(Maximum.entry()).unwrap();
        assert_eq!(Maximum.parse(index, &url).links.len(), 1);
    }
}
