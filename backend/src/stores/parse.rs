//! Parsing helpers shared by all store adapters.

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

/// The price an element shows. Some stores nest the crossed-out old price inside it,
/// so the element's own text counts first and its children only when it has none.
pub fn price_in(element: ElementRef) -> Option<Bani> {
    let own: String = element.children().filter_map(|node| node.value().as_text().map(|text| &**text)).collect();
    price(&own).or_else(|| price(&text(element)))
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
}
