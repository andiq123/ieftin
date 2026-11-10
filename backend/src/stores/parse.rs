//! Parsing helpers shared by all store adapters.

use std::collections::HashSet;

use scraper::ElementRef;
use url::Url;

use crate::domain::Bani;

/// A lazily compiled, statically cached CSS selector.
macro_rules! selector {
    ($css:literal) => {{
        static SELECTOR: std::sync::LazyLock<scraper::Selector> =
            std::sync::LazyLock::new(|| scraper::Selector::parse($css).expect("valid selector"));
        &*SELECTOR
    }};
}
pub(super) use selector;

/// Element text with whitespace collapsed.
pub fn text(element: ElementRef) -> String {
    element.text().flat_map(str::split_whitespace).collect::<Vec<_>>().join(" ")
}

/// Text of the first descendant matching `selector`, if it is not empty.
pub fn text_of(scope: ElementRef, selector: &scraper::Selector) -> Option<String> {
    scope.select(selector).next().map(text).filter(|text| !text.is_empty())
}

/// Text from a data field that may carry HTML entities or tags; `None` when empty.
pub fn plain_text(raw: &str) -> Option<String> {
    let text = text(scraper::Html::parse_fragment(raw).root_element());
    (!text.is_empty()).then_some(text)
}

/// Whether a fetched document is a sitemap rather than an HTML page.
pub fn is_sitemap(body: &str) -> bool {
    let head = &body[..body.floor_char_boundary(600)];
    head.contains("<urlset") || head.contains("<sitemapindex")
}

/// Resolves an element's URL attribute against the page it was found on.
pub fn link(element: ElementRef, attribute: &str, page: &Url) -> Option<Url> {
    page.join(element.attr(attribute)?.trim()).ok()
}

/// Parses a displayed price such as "30 999 lei", "1 299,50 lei" or "18.999 MDL".
pub fn price(text: &str) -> Option<Bani> {
    let number: String = text.chars().filter(|c| c.is_ascii_digit() || matches!(c, '.' | ',')).collect();
    let number = number.trim_matches(['.', ',']);
    // The last separator is a decimal point only when one or two digits follow it;
    // otherwise every separator groups thousands.
    let (whole, fraction) = match number.rsplit_once(['.', ',']) {
        Some((whole, fraction)) if fraction.len() <= 2 => (whole, fraction),
        _ => (number, ""),
    };
    let whole: String = whole.chars().filter(char::is_ascii_digit).collect();
    let lei: f64 = format!("{whole}.{fraction}0").parse().ok()?;
    let bani = (lei * 100.0).round() as Bani;
    (bani > 0).then_some(bani)
}

/// The `<loc>` entries of a sitemap.
pub fn sitemap_locs(xml: &str) -> impl Iterator<Item = Url> + '_ {
    xml.split("<loc>")
        .skip(1)
        .filter_map(|rest| rest.split("</loc>").next())
        .map(|loc| loc.trim().trim_start_matches("<![CDATA[").trim_end_matches("]]>").replace("&amp;", "&"))
        .filter_map(|loc| Url::parse(loc.trim()).ok())
}

/// Keeps only the deepest categories: a parent lists the same products as its children.
pub fn leaves(urls: impl IntoIterator<Item = Url>) -> Vec<Url> {
    let path = |url: &Url| url.path().trim_end_matches('/').to_owned();
    let urls: Vec<Url> = urls.into_iter().collect();
    let parents: HashSet<String> = urls
        .iter()
        .flat_map(|url| {
            let path = path(url);
            path.match_indices('/').map(|(i, _)| path[..i].to_owned()).collect::<Vec<_>>()
        })
        .collect();
    let mut seen = HashSet::new();
    urls.into_iter().filter(|url| !parents.contains(&path(url)) && seen.insert(path(url))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prices() {
        assert_eq!(price(" 30\u{a0}999 lei "), Some(3_099_900));
        assert_eq!(price("1 299,50 lei"), Some(129_950));
        assert_eq!(price("18.999 MDL"), Some(1_899_900));
        assert_eq!(price("5 967.96 MDL"), Some(596_796));
        assert_eq!(price("1.234.567"), Some(123_456_700));
        assert_eq!(price("lei"), None);
    }

    #[test]
    fn keeps_only_leaf_categories() {
        let urls = ["https://x.md/a", "https://x.md/a/b", "https://x.md/a-b", "https://x.md/a/b/"]
            .map(|url| Url::parse(url).unwrap());
        let kept: Vec<_> = leaves(urls).iter().map(|url| url.path().to_owned()).collect();
        assert_eq!(kept, ["/a/b", "/a-b"]);
    }
}
