use scraper::{ElementRef, Html};
use url::Url;

use super::{
    Store, StoreInfo,
    parse::{self, selector},
};
use crate::domain::{Page, ScrapedOffer};

/// Atehno's sitemap has not been regenerated since 2022 and its search is closed to
/// robots, so the catalogue is read from the category listings linked on the home page.
pub struct Atehno;

impl StoreInfo for Atehno {
    fn id(&self) -> &'static str {
        "atehno"
    }

    fn name(&self) -> &'static str {
        "Atehno"
    }

    fn home(&self) -> &'static str {
        "https://atehno.md"
    }
}

impl Store for Atehno {
    fn entry(&self) -> &'static str {
        "https://atehno.md/"
    }

    fn parse(&self, body: &str, url: &Url) -> Page {
        let document = Html::parse_document(body);
        if page_number(url).is_none() {
            // The home page: its menu links every category.
            let categories = document.select(selector!("a[href*='catalog/']"));
            let links = categories.filter_map(|anchor| parse::link(anchor, "href", url)).filter(is_category).collect();
            return Page { offers: Vec::new(), links };
        }
        // The heading also holds a product count, after the name.
        let category = document
            .select(selector!("h1.catalog-title"))
            .next()
            .and_then(|heading| heading.text().map(str::trim).find(|text| !text.is_empty()));
        let next = page_number(url).map(|page| page + 1);
        Page {
            offers: document
                .select(selector!("article.product-item"))
                .filter_map(|card| offer(card, category, url))
                .collect(),
            links: document
                .select(selector!("a[data-ci-pagination-page]"))
                .filter_map(|anchor| parse::link(anchor, "href", url))
                .find(|link| page_number(link) == next)
                .into_iter()
                .collect(),
        }
    }
}

/// A listing is `/catalog/<category>`; filtered and sorted views of it are skipped.
fn is_category(url: &Url) -> bool {
    url.query().is_none() && page_number(url) == Some(1)
}

/// The page of a category listing (`/catalog/<category>/page/<n>`, the first without
/// the suffix); `None` for anything else.
fn page_number(url: &Url) -> Option<u32> {
    let segments: Vec<&str> = url.path().split('/').filter(|segment| !segment.is_empty()).collect();
    match segments[..] {
        ["catalog", _] => Some(1),
        ["catalog", _, "page", page] => page.parse().ok(),
        _ => None,
    }
}

fn offer(card: ElementRef, category: Option<&str>, page: &Url) -> Option<ScrapedOffer> {
    let anchor = card.select(selector!("h3 a")).next()?;
    let url = parse::link(anchor, "href", page)?;
    let price_of = |selector| parse::text_of(card, selector).as_deref().and_then(parse::price);
    let in_stock = if card.select(selector!(".item-available")).next().is_some() {
        Some(true)
    } else if card.value().classes().any(|class| class == "product-invisible") {
        Some(false)
    } else {
        // "On order": it can be bought, but is not on the shelf.
        None
    };

    Some(ScrapedOffer {
        // Product URLs end in the article number, which outlives a renamed product.
        external_id: url.path().rsplit(['-', '/']).next().filter(|id| !id.is_empty())?.to_owned(),
        title: Some(parse::text(anchor)).filter(|title| !title.is_empty())?,
        image_url: card.select(selector!("a.img-wr img")).next().and_then(|image| parse::link(image, "src", page)),
        category: category.map(str::to_owned),
        price: price_of(selector!(".price ins .amount"))?,
        old_price: price_of(selector!(".price del .amount")),
        in_stock,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = r#"
        <h1 class="catalog-title">
            Apple <span class="catalog-count">66 моделей</span>
        </h1>
        <article class="product-item ">
          <div class="product-available-label item-available" title="В наличии"></div>
          <a class="img-wr" href="https://atehno.md/products/iphone-18-pro-256gb-black-5g-9857902828000">
            <img src="https://atehno.md/files/items/thumb_500x500_iphone.png" class="img-responsive" alt=""></a>
          <span class="price clearfix">
            <del> <span class="amount">30999<span class="currency">лей </span></span> </del>
            <ins><span class="amount">29354<span class="currency">лей </span></span></ins>
          </span>
          <h3><a href="https://atehno.md/products/iphone-18-pro-256gb-black-5g-9857902828000">
            <span style="color: #1072e3"> </span>
            iPhone 18 Pro 256GB Black 5G </a></h3>
        </article>
        <article class="product-item  product-invisible">
          <span class="price clearfix">
            <ins><span class="amount">28599<span class="currency">лей </span></span></ins>
          </span>
          <h3><a href="https://atehno.md/products/iphone-17-pro-max-256gb-silver-9857902469111">
            iPhone 17 Pro Max 256GB Silver 5G</a></h3>
        </article>
        <ul class="pagination"><li class="active"><span>1</span></li>
          <li><a href="https://atehno.md/catalog/apple-494/page/2" data-ci-pagination-page="2">2</a></li>
          <li><a href="https://atehno.md/catalog/apple-494/page/9" data-ci-pagination-page="9">»</a></li></ul>"#;

    #[test]
    fn parses_listing() {
        let page = Atehno.parse(LISTING, &Url::parse("https://atehno.md/catalog/apple-494").unwrap());
        assert_eq!(page.links.iter().map(Url::path).collect::<Vec<_>>(), ["/catalog/apple-494/page/2"]);
        let [offer, sold_out] = page.offers.as_slice() else { panic!("expected two offers") };
        assert_eq!(offer.external_id, "9857902828000");
        assert_eq!(offer.title, "iPhone 18 Pro 256GB Black 5G");
        assert_eq!(offer.category.as_deref(), Some("Apple"));
        assert_eq!((offer.price, offer.old_price, offer.in_stock), (2_935_400, Some(3_099_900), Some(true)));
        assert_eq!(offer.image_url.as_ref().unwrap().path(), "/files/items/thumb_500x500_iphone.png");
        assert_eq!((sold_out.price, sold_out.old_price, sold_out.in_stock), (2_859_900, None, Some(false)));
    }

    #[test]
    fn follows_the_next_page_only() {
        let page = Atehno.parse(LISTING, &Url::parse("https://atehno.md/catalog/apple-494/page/8").unwrap());
        assert_eq!(page.links.iter().map(Url::path).collect::<Vec<_>>(), ["/catalog/apple-494/page/9"]);
    }

    #[test]
    fn finds_categories_on_the_home_page() {
        let home = r#"<base href="https://atehno.md/"><a href="catalog/mobile-phones-490">Phones</a>
            <a href="https://atehno.md/catalog/pc-684/filter/foptional_actional">Sale</a>
            <a href="https://atehno.md/catalog/charging-stations-13356?sort_by=popular-desc">Popular</a>
            <a href="spage/catalog/kits-11669">Kits</a>
            <a href="https://atehno.md/products/iphone-17-256gb-black-9857902469463">iPhone</a>"#;
        let links = Atehno.parse(home, &Url::parse(Atehno.entry()).unwrap()).links;
        assert_eq!(links.iter().map(Url::path).collect::<Vec<_>>(), ["/catalog/mobile-phones-490"]);
    }
}
