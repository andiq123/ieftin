//! Every known store. Whether one actually works is decided by its daily health
//! check, not here, so a store that fixes (or breaks) its site is picked up automatically.

use std::sync::Arc;

use super::{
    LiveStore, Store, StoreInfo,
    atehno::Atehno,
    darwin::{DarwinPlatform, DarwinSearch},
    live::CssSearch,
    maximum::Maximum,
    structured::Structured,
};

pub struct Registry {
    /// Stores queried through their own search page at search time.
    pub live: Vec<Arc<dyn LiveStore>>,
    /// Stores crawled daily into the index, because they forbid live search.
    pub indexed: Vec<Arc<dyn Store>>,
}

impl Registry {
    /// Every store with whether it is searched live.
    pub fn stores(&self) -> impl Iterator<Item = (&dyn StoreInfo, bool)> {
        let live = self.live.iter().map(|store| (store.as_ref() as &dyn StoreInfo, true));
        let indexed = self.indexed.iter().map(|store| (store.as_ref() as &dyn StoreInfo, false));
        live.chain(indexed)
    }
}

/// Stores whose search results are plain HTML cards.
const LIVE: [CssSearch; 8] = [
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
];

pub fn registry() -> Registry {
    let mut live: Vec<Arc<dyn LiveStore>> = vec![Arc::new(DarwinSearch)];
    live.extend(LIVE.into_iter().map(|store| Arc::new(store) as Arc<dyn LiveStore>));

    // Stores on the Darwin e-commerce platform, crawled through their category listings.
    let platform = [
        DarwinPlatform::new(
            "enter",
            "Enter",
            "https://enter.online",
            "https://enter.online/storage/sitemaps/categories.xml",
        ),
        DarwinPlatform::new("istore", "iStore", "https://istore.md", "https://istore.md/media/sitemaps/categories.xml"),
    ];

    // Stores read through the structured product data on their product pages.
    let structured = [
        Structured::new("alo", "Alo", "https://alo.md", "https://alo.md/sitemap.xml")
            .sitemaps(&["category-products/"])
            .pages(&["/ro/product/"]),
        Structured::new("ultra", "Ultra", "https://ultra.md", "https://ultra.md/sitemap/sitemap-ro.xml")
            .sitemaps(&["products-ro"])
            .pages(&["/product/"]),
        Structured::new("nanoteh", "Nanoteh", "https://nanoteh.md", "https://nanoteh.md/sitemap-index.xml")
            .sitemaps(&["sitemap-products-ro-"]),
        Structured::new(
            "moldcell",
            "Moldcell",
            "https://eshop.moldcell.md",
            "https://eshop.moldcell.md/sitemap-ro.xml",
        )
        .pages(&["/ro/"]),
        Structured::new("hitech", "Hi-Tech", "https://h-t.md", "https://h-t.md/sitemap_main.xml").pages(&["/md/"]),
        Structured::new(
            "neocomputer",
            "NeoComputer",
            "https://neocomputer.md",
            "https://neocomputer.md/sitemap-products.xml",
        )
        .skip(&["/ru/"]),
        Structured::new("ispace", "iSpace", "https://ispace.md", "https://ispace.md/sitemap.xml")
            .pages(&["/product/"])
            .skip(&["/ru/"]),
        Structured::new("topsale", "TopSale", "https://topsale.md", "https://topsale.md/topsale-sitemap.xml")
            .sitemaps(&["sitemap-products"])
            .pages(&["/ro-"]),
        Structured::new("xiaomistore", "Xiaomi Store", "https://xiaomistore.md", "https://xiaomistore.md/sitemap.xml")
            .sitemaps(&["category-products/"])
            .pages(&["/ro/product/"]),
        Structured::new(
            "titan",
            "Titan Electronic",
            "https://titanelectronic.md",
            "https://titanelectronic.md/sitemap.xml",
        )
        .sitemaps(&["catalog-sitemap"])
        .skip(&["/ru/"]),
        Structured::new("doxyterra", "Doxyterra", "https://doxyterra.md", "https://doxyterra.md/sitemap.xml")
            .sitemaps(&["catalog-sitemap"])
            .pages(&["/ro/"]),
        Structured::new("allgames", "AllGames", "https://allgames.md", "https://allgames.md/sitemap.xml")
            .sitemaps(&["product-sitemap"])
            .pages(&["/product/"]),
        Structured::new("microclima", "Microclima", "https://microclima.md", "https://microclima.md/sitemap_index.xml")
            .sitemaps(&["product-sitemap"])
            .pages(&["/produs/"]),
        Structured::new("agropiese", "Agropiese", "https://agropiese.md", "https://agropiese.md/sitemap_products.xml")
            .pages(&["/catalog/"]),
        // These fail their health check today: no machine-readable prices, or automated
        // access refused. They stay listed so the check keeps trying and the status shows.
        Structured::new("xstore", "Xstore", "https://xstore.md", "https://xstore.md/sitemap.xml")
            .sitemaps(&["sitemaps/products-"])
            .skip(&["/ru/"]),
        Structured::new("texet", "Texet", "https://www.texet.md", "https://www.texet.md/sitemap.xml")
            .sitemaps(&["sitemap/products/"])
            .pages(&["/ro/product/"]),
        Structured::new("smart", "Smart", "https://www.smart.md", "https://www.smart.md/sitemaps/sitemap.xml")
            .sitemaps(&["categorized-products"])
            .skip(&["/ru/", "-ru-"]),
        Structured::new("bigshop", "BigShop", "https://bigshop.md", "https://bigshop.md/sitemap-index.xml")
            .sitemaps(&["sitemap-products-"])
            .pages(&["/ro/product/"]),
        Structured::new("primepc", "Prime PC", "https://prime-pc.md", "https://prime-pc.md/sitemap.xml")
            .pages(&["/products/"]),
        Structured::new("technomir", "Technomir", "https://technomir.md", "https://technomir.md/sitemap.xml")
            .sitemaps(&["sitemap_products_"])
            .pages(&["/product/"]),
        Structured::new(
            "exit",
            "Exit Accessories",
            "https://exitaccessories.md",
            "https://exitaccessories.md/sitemap-index.xml",
        )
        .sitemaps(&["sitemap-catalog-"])
        .pages(&["/product/"]),
        Structured::new("tehnoparc", "Tehnoparc", "https://tehnoparc.md", "https://tehnoparc.md/ru/sitemap.xml")
            .sitemaps(&["sitemap-iblock-"]),
        Structured::new(
            "pandashop",
            "PandaShop",
            "https://www.pandashop.md",
            "https://www.pandashop.md/sitemaps/categories-ro.xml",
        )
        .pages(&["/ro/catalog/"]),
        Structured::new("coleso", "Coleso", "https://coleso.md", "https://coleso.md/sitemap.xml"),
        Structured::new("compas", "Compas", "https://compas.md", "https://compas.md/sitemap.xml"),
        Structured::new("orange", "Orange", "https://eshop.orange.md", "https://eshop.orange.md/sitemap.xml"),
    ];

    let mut indexed: Vec<Arc<dyn Store>> = vec![Arc::new(Maximum), Arc::new(Atehno)];
    indexed.extend(platform.into_iter().map(|store| Arc::new(store) as Arc<dyn Store>));
    indexed.extend(structured.into_iter().map(|store| Arc::new(store) as Arc<dyn Store>));
    Registry { live, indexed }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use url::Url;

    use super::*;

    #[test]
    fn registry_is_consistent() {
        let registry = registry();
        let ids: HashSet<_> = registry.stores().map(|(store, _)| store.id()).collect();
        assert_eq!(ids.len(), registry.stores().count(), "store ids must be unique");

        for store in &registry.indexed {
            let home = Url::parse(store.home()).unwrap();
            assert_eq!(home.host_str(), Url::parse(store.entry()).unwrap().host_str(), "{}", store.id());
        }
        for store in &registry.live {
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
        let registry = registry();
        let store = registry.live.iter().find(|store| store.id() == "gsmshop").unwrap();
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
