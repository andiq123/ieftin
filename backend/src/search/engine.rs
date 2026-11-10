//! Turns every matching offer into one page of results. Pure functions: where the
//! offers came from (index or live) no longer matters here.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::TextQuery;
use crate::domain::{Bani, Offer};

const MAX_FACET_VALUES: usize = 24;
/// The product searched for costs at least this many times more than its accessories.
const ACCESSORY_RATIO: Bani = 3;
/// How far below a product category's cheapest offer the floor is set.
const CATEGORY_MARGIN_PERCENT: Bani = 80;
const BANI_PER_LEU: Bani = 100;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    PriceAsc,
    PriceDesc,
}

#[derive(Debug, Default)]
pub struct Filters {
    pub store: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<Bani>,
    pub max_price: Option<Bani>,
    pub in_stock_only: bool,
    /// Show everything, even offers below the detected accessory floor.
    pub include_accessories: bool,
    pub sort: Sort,
    pub limit: usize,
    pub offset: usize,
}

/// How many matches share a store or category, and the cheapest of them.
#[derive(Debug, Serialize, PartialEq)]
pub struct Facet {
    pub key: String,
    pub count: usize,
    pub min_price_bani: Bani,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub total: usize,
    pub offers: Vec<Offer>,
    /// Facets describe every match, ignoring the other filters, so the user can
    /// always see (and switch to) the alternatives.
    pub stores: Vec<Facet>,
    pub categories: Vec<Facet>,
    /// Price below which matches look like accessories or related items (cases,
    /// games, cables…) rather than the product searched for. Offers under it are
    /// left out unless asked for; `None` when there is no clear split.
    pub accessory_floor_bani: Option<Bani>,
}

pub fn assemble(mut offers: Vec<Offer>, text: &TextQuery, filters: &Filters) -> Results {
    // The same product page can arrive twice (e.g. listed in two categories).
    let mut seen = HashSet::new();
    offers.retain(|offer| seen.insert(offer.url.clone()));

    let stores = facets(&offers, |offer| Some(&offer.store_id));
    let categories = facets(&offers, |offer| offer.category.as_ref());

    // An explicit category or minimum price already says what the user wants.
    let accessory_floor_bani = match (&filters.category, filters.min_price) {
        (None, None) => accessory_floor(&offers, text.words()),
        _ => None,
    };
    let min_price = match accessory_floor_bani {
        Some(floor) if !filters.include_accessories => Some(floor),
        _ => filters.min_price,
    };

    offers.retain(|offer| {
        filters.store.as_ref().is_none_or(|store| &offer.store_id == store)
            && filters.category.as_ref().is_none_or(|category| offer.category.as_ref() == Some(category))
            && min_price.is_none_or(|min| offer.price_bani >= min)
            && filters.max_price.is_none_or(|max| offer.price_bani <= max)
            && !(filters.in_stock_only && offer.in_stock == Some(false))
    });
    match filters.sort {
        Sort::PriceAsc => offers.sort_by(|a, b| (a.price_bani, &a.url).cmp(&(b.price_bani, &b.url))),
        Sort::PriceDesc => offers.sort_by(|a, b| (b.price_bani, &a.url).cmp(&(a.price_bani, &b.url))),
    }

    Results {
        total: offers.len(),
        offers: offers.into_iter().skip(filters.offset).take(filters.limit).collect(),
        stores,
        categories,
        accessory_floor_bani,
    }
}

fn facets<'a>(offers: &'a [Offer], key: impl Fn(&'a Offer) -> Option<&'a String>) -> Vec<Facet> {
    let mut groups: HashMap<&str, (usize, Bani)> = HashMap::new();
    for offer in offers {
        if let Some(key) = key(offer) {
            let (count, min_price) = groups.entry(key).or_insert((0, Bani::MAX));
            *count += 1;
            *min_price = (*min_price).min(offer.price_bani);
        }
    }
    let mut facets: Vec<Facet> = groups
        .into_iter()
        .map(|(key, (count, min_price_bani))| Facet { key: key.to_owned(), count, min_price_bani })
        .collect();
    facets.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    facets.truncate(MAX_FACET_VALUES);
    facets
}

/// Where the searched product's prices start, judged two ways. Categories are the
/// better signal but not every offer has one, so the price gap is the fallback.
fn accessory_floor(offers: &[Offer], words: usize) -> Option<Bani> {
    // A single word ("samsung") names no particular product to separate out.
    let by_category = if words >= 2 { category_floor(offers) } else { None };
    by_category.or_else(|| gap_floor(offers))
}

/// The middle value of a non-empty, sorted list.
fn median(sorted: &[Bani]) -> Bani {
    sorted[sorted.len() / 2]
}

/// Searching "playstation 5" matches a few consoles and many games. Among the offers
/// that have a category, one whose typical price towers over the typical offer holds
/// the product itself. The floor sits a margin below the cheapest offer of such a
/// category, because another store may sell the same product for less. Being a
/// price, it then applies to every offer, including those without a category.
fn category_floor(offers: &[Offer]) -> Option<Bani> {
    let mut categories: HashMap<&str, Vec<Bani>> = HashMap::new();
    for offer in offers {
        if let Some(category) = &offer.category {
            categories.entry(category).or_default().push(offer.price_bani);
        }
    }
    let mut categorised: Vec<Bani> = categories.values().flatten().copied().collect();
    categorised.sort_unstable();
    let typical = median(categorised.get(..).filter(|prices| !prices.is_empty())?);

    let significant = (categorised.len() / 20).max(3);
    let cheapest = categories.into_values().filter_map(|mut prices| {
        prices.sort_unstable();
        (prices.len() >= significant && median(&prices) >= typical * ACCESSORY_RATIO).then(|| prices[0])
    });
    // Rounded down to whole lei, since the floor is shown to the user.
    cheapest.min().map(|price| price * CATEGORY_MARGIN_PERCENT / 100 / BANI_PER_LEU * BANI_PER_LEU)
}

/// Without categories: searching "iphone 15 pro" matches 200-lei cases and
/// 20 000-lei phones with nothing in between; the upper edge of the widest relative
/// gap is the floor. The upper group must be a real share of the matches, so a single
/// expensive outlier never hides everything else.
fn gap_floor(offers: &[Offer]) -> Option<Bani> {
    let mut prices: Vec<Bani> = offers.iter().map(|offer| offer.price_bani).collect();
    prices.sort_unstable();
    let significant = (prices.len() * 15 / 100).max(3);
    (1..prices.len())
        .filter(|&i| prices[i] >= prices[i - 1] * ACCESSORY_RATIO && prices.len() - i >= significant)
        .max_by(|&a, &b| {
            let ratio = |i: usize| prices[i] as f64 / prices[i - 1] as f64;
            ratio(a).total_cmp(&ratio(b))
        })
        .map(|i| prices[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(store: &str, title: &str, lei: Bani, category: Option<&str>) -> Offer {
        Offer {
            store_id: store.into(),
            external_id: title.into(),
            title: title.into(),
            url: format!("https://{store}.md/{title}/{lei}"),
            image_url: None,
            category: category.map(Into::into),
            price_bani: lei * 100,
            old_price_bani: None,
            in_stock: None,
            live: false,
        }
    }

    fn results(offers: Vec<Offer>, text: &str, filters: Filters) -> Results {
        assemble(offers, &TextQuery::parse(text).unwrap(), &Filters { limit: 100, ..filters })
    }

    #[test]
    fn hides_accessories_below_a_price_gap() {
        let mut offers: Vec<_> = (0..6).map(|i| offer("a", "case", 150 + i * 20, None)).collect();
        offers.extend((0..4).map(|i| offer("b", "phone", 17_000 + i * 500, None)));

        let hidden = results(offers.clone(), "iphone 15 pro", Filters::default());
        assert_eq!((hidden.accessory_floor_bani, hidden.total), (Some(1_700_000), 4));
        assert_eq!(hidden.offers[0].price_bani, 1_700_000);

        let shown = results(offers, "iphone 15 pro", Filters { include_accessories: true, ..Filters::default() });
        assert_eq!(shown.total, 10);
    }

    #[test]
    fn uses_categories_when_prices_have_no_gap() {
        // Games, controllers and consoles form a continuous range of prices.
        let mut offers: Vec<_> = (0..40).map(|i| offer("a", "game", 200 + i * 60, Some("Jocuri"))).collect();
        offers.extend((0..6).map(|i| offer("a", "pad", 1_500 + i * 500, Some("Accesorii"))));
        offers.extend((0..5).map(|i| offer("a", "console", 9_500 + i * 1_500, Some("Console"))));
        offers.push(offer("b", "console elsewhere", 9_900, None));

        let found = results(offers.clone(), "playstation 5", Filters::default());
        assert_eq!((found.accessory_floor_bani, found.total), (Some(760_000), 6));
        // One word is too vague to single out a product.
        assert_eq!(results(offers, "playstation", Filters::default()).accessory_floor_bani, None);
    }

    #[test]
    fn filters_sorts_pages_and_counts() {
        let offers = vec![
            offer("a", "x", 300, Some("TV")),
            offer("b", "y", 100, Some("TV")),
            offer("b", "z", 200, None),
            offer("b", "z", 200, None), // duplicate URL
        ];
        let all = results(offers.clone(), "tv", Filters::default());
        assert_eq!(all.total, 3);
        assert_eq!(all.stores[0], Facet { key: "b".into(), count: 2, min_price_bani: 10_000 });
        assert_eq!(all.offers.iter().map(|o| o.price_bani).collect::<Vec<_>>(), [10_000, 20_000, 30_000]);

        let filtered = results(
            offers,
            "tv",
            Filters { store: Some("b".into()), sort: Sort::PriceDesc, offset: 1, ..Filters::default() },
        );
        assert_eq!((filtered.total, filtered.offers.len(), filtered.offers[0].price_bani), (2, 1, 10_000));
    }
}
