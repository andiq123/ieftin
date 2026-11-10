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

/// Things sold *for* a product rather than the product itself. A listing that names
/// one of these is an accessory, wherever the word stands in its title. Romanian and
/// English as stores write them, and Russian as [`normalize`] transliterates it.
const ACCESSORY_WORDS: &[&str] = &[
    // Cases and covers.
    "husa",
    "huse",
    "case",
    "cases",
    "cover",
    "carcasa",
    "bumper",
    "toc",
    "chekhol",
    "chekhly",
    "nakladka",
    // Screen protection.
    "sticla",
    "sticle",
    "folie",
    "folii",
    "protector",
    "protectie",
    "glass",
    "steklo",
    "zashchitnoe",
    "plenka",
    "plionka",
    // Power and cables.
    "cablu",
    "cabluri",
    "cable",
    "kabel",
    "incarcator",
    "charger",
    "zariadnoe",
    "adaptor",
    "adapter",
    "alimentator",
    "acumulator",
    "baterie",
    "battery",
    "akkumuliator",
    "batareia",
    // Carrying and holding.
    "suport",
    "holder",
    "derzhatel",
    "podstavka",
    "curea",
    "strap",
    "bratara",
    "remeshok",
    "geanta",
    "rucsac",
    "sumka",
    "riukzak",
    // Small extras.
    "stylus",
    "skin",
    "sticker",
    "stickere",
];
/// Words that mark an accessory only when a title opens with them: "Joc PS5 …" is a
/// game, while "PlayStation 5 + joc" is a console sold with one.
const ACCESSORY_LEADS: &[&str] = &["joc", "jocuri", "game", "games", "igra", "igry"];

/// Whether a title (already passed through [`normalize`]) names an accessory.
pub fn names_accessory(normalized_title: &str) -> bool {
    let mut words = normalized_title.split(' ');
    words.next().is_some_and(|first| ACCESSORY_LEADS.contains(&first) || ACCESSORY_WORDS.contains(&first))
        || words.any(|word| ACCESSORY_WORDS.contains(&word))
}

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

    /// Whether the user asked for an accessory ("husa iphone 15"): then accessories
    /// are the product, and nothing should be set aside.
    pub fn wants_accessory(&self) -> bool {
        self.terms
            .iter()
            .any(|term| ACCESSORY_WORDS.contains(&term.word.as_str()) || ACCESSORY_LEADS.contains(&term.word.as_str()))
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
    fn tells_accessories_from_products() {
        let accessory = |title: &str| names_accessory(&normalize(title));
        assert!(accessory("Husă pentru smartphone Apple iPhone 15 Pro Max (Nature Pro) White"));
        assert!(accessory("Original iPhone 15 Pro Silicone Case Black"));
        assert!(accessory("Sticla de pro tectie HELMET DIAMOND 5D IPHONE 15 PRO , BLACK"));
        assert!(accessory("Joc PS5 EA Sports FC 25"));
        assert!(accessory("Acumulator Hoco Battery J112 Pro for Apple iPhone 15 Pro"));
        // Russian titles, as some stores write them.
        assert!(accessory("Чехол Helmet TPU Soft iPhone 15 Pro Clear"));
        assert!(accessory("Защитное стекло для iPhone 15 Pro"));
        assert!(!accessory("Смартфон Apple iPhone 15 128GB Black"));
        assert!(!accessory("Telefon mobil iPhone 15 Pro 1TB Blue Titanium"));
        assert!(!accessory("Consolă Sony PlayStation 5 Slim + joc EA FC 25"));
        assert!(!accessory("Laptop gaming ASUS ROG Strix"));

        assert!(query("husa iphone 15").wants_accessory());
        assert!(!query("iphone 15 pro").wants_accessory());
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
