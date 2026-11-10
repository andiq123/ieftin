//! Every known store. Whether one actually works is decided by its health check, not
//! here, so a store that fixes (or breaks) its site is picked up automatically.

use super::{Store, css::CssSearch, darwin::Darwin};

/// Stores whose search results are plain HTML cards.
const CSS: [CssSearch; 15] = [
    CssSearch {
        id: "bomba",
        name: "Bomba",
        home: "https://bomba.md",
        search: "https://bomba.md/ro/cautare/?query={}",
        probe: "samsung",
        card: ".card-product-item",
        link: ".product-info a",
        title: None,
        price: ".product-price .price",
        old_price: Some(".product-price .old-price"),
        image: Some((".product-photo img", "src")),
    },
    CssSearch {
        id: "cactus",
        name: "Cactus",
        home: "https://www.cactus.md",
        search: "https://www.cactus.md/ro/search/?q={}",
        probe: "samsung",
        card: ".catalog__pill",
        link: "a[href*='/catalogue/']",
        title: Some(".catalog__pill__text__title"),
        price: ".catalog__pill__controls__price:not(.catalog__pill__controls__price_old)",
        old_price: Some(".catalog__pill__controls__price_old"),
        image: Some(("img.catalog__pill__img__prime", "src")),
    },
    CssSearch {
        id: "gorilla",
        name: "Gorilla",
        home: "https://gorilla.md",
        search: "https://gorilla.md/?s={}&post_type=product",
        probe: "samsung",
        card: ".product-wrapper",
        link: ".product-title a",
        title: None,
        price: ".price ins bdi, .price > .amount bdi",
        old_price: Some(".price del bdi"),
        image: Some((".product-card img", "src")),
    },
    CssSearch {
        id: "fantastic",
        name: "Fantastic",
        home: "https://www.fantastic.md",
        search: "https://www.fantastic.md/catalog?search={}",
        probe: "samsung",
        card: ".Content_Goods_ItemBlock",
        link: ".Content_Goods_TitleBlock a",
        title: None,
        price: ".Content_Goods_PriceBlock_Value_lei",
        old_price: None,
        image: Some((".Content_Goods_ItemBlock_ImageArea a", "data-src")),
    },
    CssSearch {
        id: "flexmag",
        name: "FlexMag",
        home: "https://flexmag.md",
        search: "https://flexmag.md/cautare/?search={}",
        probe: "bosch",
        card: ".product-card",
        link: "a[href*='/produs/']",
        title: Some(".product-title"),
        price: ".actual-price p",
        old_price: None,
        image: Some(("img.product-img", "src")),
    },
    CssSearch {
        id: "smadshop",
        name: "SmadShop",
        home: "https://smadshop.md",
        search: "https://smadshop.md/ro/search/?search={}",
        probe: "samsung",
        card: ".product-wrap",
        link: ".product-name a",
        title: None,
        price: ".new-price",
        old_price: Some(".old-price"),
        image: Some((".product-media img", "src")),
    },
    CssSearch {
        id: "masterlux",
        name: "Master-Lux",
        home: "https://master-lux.md",
        search: "https://master-lux.md/search?search={}",
        probe: "michelin",
        card: ".product-card-item",
        link: ".card > a[href]",
        title: Some(".card-title"),
        price: "p.price",
        old_price: None,
        image: Some((".card > a img", "src")),
    },
    CssSearch {
        id: "gsmshop",
        name: "GSM Shop",
        home: "https://www.gsmshop.md",
        search: "https://www.gsmshop.md/ro/product/search.html?search={}",
        probe: "samsung",
        card: ".item-card",
        link: ".name-item a",
        title: None,
        // Sold-out products have no price and are left out.
        price: ".price span:first-child",
        old_price: None,
        image: Some((".image-item img", "src")),
    },
    CssSearch {
        id: "smart",
        name: "Smart",
        home: "https://www.smart.md",
        search: "https://www.smart.md/s?query={}",
        probe: "samsung",
        card: ".category-product-card",
        link: ".custom_product_title a",
        title: None,
        price: ".custom_product_price .regular",
        old_price: Some(".custom_product_price .special"),
        image: Some((".custom_product_image_default img", "src")),
    },
    CssSearch {
        id: "xstore",
        name: "Xstore",
        home: "https://xstore.md",
        search: "https://xstore.md/search?search={}",
        probe: "samsung",
        card: "figure.card-product",
        link: "a.xnc-title",
        // The model and its variant ("6 GB / 128 GB / Blue") are separate elements.
        title: Some(".xnc-title, .xnc-attr"),
        price: ".xnc-price",
        old_price: Some(".xnc-price-striked"),
        image: Some((".xnc-img", "src")),
    },
    CssSearch {
        id: "pandashop",
        name: "PandaShop",
        home: "https://www.pandashop.md",
        search: "https://www.pandashop.md/ro/search/?text={}",
        probe: "samsung",
        card: ".js-itemsList-item",
        link: "a.card-title",
        title: None,
        price: ".card-price_curr",
        old_price: None,
        image: Some((".card-img-picture img", "data-src")),
    },
    CssSearch {
        id: "primepc",
        name: "Prime PC",
        home: "https://prime-pc.md",
        search: "https://prime-pc.md/products?keyword={}",
        probe: "samsung",
        card: "div.product",
        link: "a",
        title: Some(".product_name h3"),
        price: ".product_price",
        old_price: None,
        image: Some((".carousel-item.active img", "src")),
    },
    CssSearch {
        id: "istore",
        name: "iStore",
        home: "https://istore.md",
        search: "https://istore.md/cautare?keywords={}",
        probe: "iphone",
        card: ".card-item",
        link: "a.product-link",
        title: None,
        price: "a.product-link ~ p.fw-600",
        old_price: Some("p.text-decoration-line-through"),
        image: Some((".block-img img", "src")),
    },
    CssSearch {
        id: "nanoteh",
        name: "Nanoteh",
        home: "https://nanoteh.md",
        search: "https://nanoteh.md/ro/catalogsearch/result/?q={}",
        probe: "samsung",
        card: ".products-grid .item",
        link: ".item-title a",
        title: None,
        price: ".special-price .price, .regular-price .price",
        old_price: Some(".old-price .price"),
        image: Some((".product-image img", "src")),
    },
    CssSearch {
        id: "texet",
        name: "Texet",
        home: "https://www.texet.md",
        search: "https://www.texet.md/ro/product/search.html?search={}",
        probe: "samsung",
        card: ".phone_bls",
        link: ".phone_bls_name a",
        title: None,
        price: ".phone_bls_price",
        old_price: None,
        image: Some((".phone_bls_img img", "src")),
    },
];

pub fn all() -> Vec<Box<dyn Store>> {
    let mut stores: Vec<Box<dyn Store>> = vec![Box::new(Darwin)];
    stores.extend(CSS.into_iter().map(|store| Box::new(store) as Box<dyn Store>));
    stores
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use url::Url;

    use super::*;

    #[test]
    fn stores_are_consistent() {
        let stores = all();
        let ids: HashSet<_> = stores.iter().map(|store| store.id()).collect();
        assert_eq!(ids.len(), stores.len(), "store ids must be unique");

        for store in &stores {
            let home = Url::parse(store.home()).unwrap();
            let search = store.search_url(store.probe()).unwrap();
            assert_eq!(home.host_str(), search.host_str(), "{}", store.id());
        }
    }

    #[test]
    fn reads_gsmshop_results() {
        let html = r#"<div class="item-card  ">
              <div class="image-item"><a href="/ro/product/view/appleiphone18pro256gbblack.html">
                <img src="/images/small/products/appleiphone18pro256gbblack/black.webp" alt="" /></a></div>
              <div class="info-item">
                <div class="name-item"><a href="/ro/product/view/appleiphone18pro256gbblack.html">
                    Apple iPhone 18 Pro 256Gb Black </a></div>
                <div class="price">
                    <span>28999</span> <span style="font-size: 14px">MDL</span>
                </div>
              </div></div>
            <div class="item-card  noItemOver ">
              <div class="name-item"><a href="/ro/product/view/appleiphone18pro1tbglacier.html">
                Apple iPhone 18 Pro 1Tb Glacier</a></div></div>"#;
        let stores = all();
        let store = stores.iter().find(|store| store.id() == "gsmshop").unwrap();
        let offers = store.parse_results(html, &store.search_url("iphone 18 pro").unwrap());
        let [offer] = offers.as_slice() else { panic!("expected one offer") };
        assert_eq!(offer.title, "Apple iPhone 18 Pro 256Gb Black");
        assert_eq!(offer.url.as_str(), "https://www.gsmshop.md/ro/product/view/appleiphone18pro256gbblack.html");
        assert_eq!(offer.external_id, "/ro/product/view/appleiphone18pro256gbblack.html");
        assert_eq!((offer.price, offer.old_price), (2_899_900, None));
        assert_eq!(
            offer.image_url.as_ref().unwrap().path(),
            "/images/small/products/appleiphone18pro256gbblack/black.webp"
        );
    }
}
