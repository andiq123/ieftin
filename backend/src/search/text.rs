//! How search text and product titles are compared: both are normalised the same way,
//! then every typed word must appear in the title.

use deunicode::deunicode;

/// Lower-cases, strips diacritics, and splits letter/digit runs ("256GB" -> "256 gb",
/// "S24" -> "s 24") so spacing differences between stores do not hide matches.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut previous: Option<char> = None;
    for c in deunicode(text).chars().flat_map(char::to_lowercase) {
        if !c.is_ascii_alphanumeric() {
            previous = None;
            continue;
        }
        match previous {
            Some(p) if p.is_ascii_digit() == c.is_ascii_digit() => {}
            _ if !out.is_empty() => out.push(' '),
            _ => {}
        }
        out.push(c);
        previous = Some(c);
    }
    out
}

/// Words shoppers use interchangeably; a query word matches any word of its group.
const SYNONYMS: [&[&str]; 4] = [
    &["ps", "playstation"],
    &["tv", "televizor", "televizoare"],
    &["laptop", "laptopuri", "notebook"],
    &["telefon", "telefoane", "smartphone"],
];

/// A user's search text.
#[derive(Debug, PartialEq)]
pub struct TextQuery {
    terms: Vec<Term>,
}

/// One typed word with its synonyms.
#[derive(Debug, PartialEq)]
struct Term {
    word: String,
    /// The word's synonym group (including itself), if it has one.
    synonyms: &'static [&'static str],
    /// The last word matches as a prefix so results appear while typing, except
    /// numbers: "iphone 1" should not match every "iPhone 15".
    prefix: bool,
}

impl TextQuery {
    /// Returns `None` for text with no searchable words.
    pub fn parse(text: &str) -> Option<Self> {
        let normalized = normalize(text);
        let words: Vec<&str> = normalized.split(' ').filter(|word| !word.is_empty()).collect();
        let last = words.len().checked_sub(1)?;
        let terms = words.iter().enumerate().map(|(i, word)| Term {
            word: (*word).to_owned(),
            synonyms: SYNONYMS.iter().find(|group| group.contains(word)).copied().unwrap_or_default(),
            prefix: i == last && !word.starts_with(|c: char| c.is_ascii_digit()),
        });
        Some(Self { terms: terms.collect() })
    }

    /// Number of words typed; one-word queries are too broad to guess intent from.
    pub fn words(&self) -> usize {
        self.terms.len()
    }

    /// The normalised text, e.g. as a cache key.
    pub fn normalized(&self) -> String {
        self.terms.iter().map(|term| term.word.as_str()).collect::<Vec<_>>().join(" ")
    }

    /// Whether a title (already passed through [`normalize`]) contains every word.
    /// The last word may be the start of one, so results appear while typing, and a
    /// word with synonyms matches any of them.
    pub fn matches(&self, normalized_title: &str) -> bool {
        let title: Vec<&str> = normalized_title.split(' ').collect();
        self.terms.iter().all(|term| {
            term.words()
                .any(|word| title.iter().any(|have| if term.prefix { have.starts_with(word) } else { *have == word }))
        })
    }
}

impl Term {
    /// The typed word, or its whole synonym group.
    fn words(&self) -> impl Iterator<Item = &str> {
        let own = self.synonyms.is_empty().then_some(self.word.as_str());
        own.into_iter().chain(self.synonyms.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(text: &str) -> TextQuery {
        TextQuery::parse(text).unwrap()
    }

    #[test]
    fn normalizes_diacritics_case_and_boundaries() {
        assert_eq!(normalize("Husă iPhone 15 Pro, 256GB"), "husa iphone 15 pro 256 gb");
        assert_eq!(normalize("Galaxy S24+"), "galaxy s 24");
    }

    #[test]
    fn counts_and_normalizes_words() {
        assert_eq!((query("iPhone 15 pro").words(), query("iPhone 15 pro").normalized()), (3, "iphone 15 pro".into()));
        assert_eq!(TextQuery::parse(" ,, "), None);
    }

    #[test]
    fn requires_every_word_with_a_prefix_on_the_last_and_synonyms() {
        let matches = |text: &str, title: &str| query(text).matches(&normalize(title));
        assert!(matches("ps5", "Consolă Sony PlayStation 5 Slim"));
        assert!(matches("iphone 15 pr", "Apple iPhone 15 Pro 256GB"));
        assert!(!matches("iphone 1", "Apple iPhone 15 Pro"));
        assert!(!matches("iphone 15 pro", "Husă Samsung Galaxy S24"));
        assert!(matches("samsung tv", "Televizor Samsung QLED 55"));
    }
}
