//! Minimal robots.txt support: the rules that apply to us and whether a path is allowed.

use std::time::Duration;

/// The rule group applying to one crawler.
#[derive(Debug, Default)]
pub struct Robots {
    /// `(allow, pattern)` in file order.
    rules: Vec<(bool, String)>,
    pub crawl_delay: Option<Duration>,
}

impl Robots {
    /// Parses the group addressed to `agent`, falling back to the `*` group.
    pub fn parse(text: &str, agent: &str) -> Self {
        let agent = agent.to_lowercase();
        let (mut ours, mut everyone) = (None, None);
        let mut group = Robots::default();
        let mut names: Vec<String> = Vec::new();
        let mut in_rules = false;

        let mut close = |names: &mut Vec<String>, group: &mut Robots| {
            let group = std::mem::take(group);
            if names.iter().any(|name| agent.contains(name.as_str())) {
                ours.get_or_insert(group);
            } else if names.iter().any(|name| name == "*") {
                everyone.get_or_insert(group);
            }
            names.clear();
        };

        for line in text.lines() {
            let line = line.split('#').next().unwrap_or_default();
            let Some((field, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match field.trim().to_lowercase().as_str() {
                "user-agent" => {
                    if in_rules {
                        close(&mut names, &mut group);
                        in_rules = false;
                    }
                    names.push(value.to_lowercase());
                }
                "allow" | "disallow" if !value.is_empty() => {
                    in_rules = true;
                    group.rules.push((field.trim().eq_ignore_ascii_case("allow"), value.to_owned()));
                }
                "crawl-delay" => {
                    in_rules = true;
                    group.crawl_delay = value.parse().ok().map(Duration::from_secs_f64);
                }
                _ => in_rules |= !names.is_empty() && field.trim().eq_ignore_ascii_case("disallow"),
            }
        }
        close(&mut names, &mut group);
        ours.or(everyone).unwrap_or_default()
    }

    /// The longest matching rule decides; with no match, or a tie, crawling is allowed.
    pub fn allows(&self, path: &str) -> bool {
        self.rules
            .iter()
            .filter(|(_, pattern)| matches(pattern, path))
            .max_by_key(|(allow, pattern)| (pattern.len(), *allow))
            .is_none_or(|(allow, _)| *allow)
    }
}

/// robots.txt patterns are prefixes where `*` matches anything and `$` anchors the end.
fn matches(pattern: &str, path: &str) -> bool {
    let (pattern, anchored) = match pattern.strip_suffix('$') {
        Some(pattern) => (pattern, true),
        None => (pattern, false),
    };
    let mut parts = pattern.split('*');
    let Some(mut rest) = path.strip_prefix(parts.next().unwrap_or_default()) else {
        return false;
    };
    let mut ends_with_part = true;
    for part in parts {
        ends_with_part = false;
        if part.is_empty() {
            continue; // trailing or doubled `*`
        }
        let Some(found) = (if anchored { rest.rfind(part) } else { rest.find(part) }) else {
            return false;
        };
        rest = &rest[found + part.len()..];
        ends_with_part = true;
    }
    !anchored || !ends_with_part || rest.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROBOTS: &str = "User-agent: GPTBot\nDisallow: /\n\n\
        User-agent: *\nDisallow: /search/\nDisallow: /*?sort=\nDisallow: /*.pdf$\nAllow: /search/help\nCrawl-delay: 2\n";

    #[test]
    fn applies_the_wildcard_group() {
        let robots = Robots::parse(ROBOTS, "IeftinBot");
        assert!(robots.allows("/product/iphone"));
        assert!(!robots.allows("/search/?q=iphone"));
        assert!(robots.allows("/search/help"));
        assert!(!robots.allows("/phones?sort=price"));
        assert!(!robots.allows("/files/manual.pdf"));
        assert!(robots.allows("/files/manual.pdf.html"));
        assert_eq!(robots.crawl_delay, Some(Duration::from_secs(2)));
    }

    #[test]
    fn prefers_its_own_group() {
        let robots = Robots::parse("User-agent: *\nDisallow:\n\nUser-agent: ieftinbot\nDisallow: /\n", "IeftinBot");
        assert!(!robots.allows("/anything"));
        assert!(Robots::parse("", "IeftinBot").allows("/anything"));
    }
}
