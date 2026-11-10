use reqwest::StatusCode;

use super::{fetcher::FetchError, walk::Walk};
use crate::domain::Health;

/// Requests a check may spend looking for a first readable price.
const BUDGET: usize = 12;

/// Verifies a store end to end the same way a crawl would: start at the entry
/// document and follow links until one page yields an offer.
pub async fn check(mut walk: Walk<'_>) -> (Health, String) {
    let (mut pages_read, mut failures) = (0, 0);
    let mut last_error = None;
    for _ in 0..BUDGET {
        match walk.next().await {
            Some(Ok(page)) if !page.offers.is_empty() => {
                return (Health::Ok, format!("price read after {} requests", pages_read + 1));
            }
            Some(Ok(_)) => pages_read += 1,
            Some(Err(error)) => {
                failures += 1;
                last_error = Some(error);
            }
            None => break,
        }
    }
    match last_error {
        // Mostly failed requests: the failure is the diagnosis, even if a sitemap loaded.
        Some(error) if failures >= pages_read => (classify(&error), error.to_string()),
        _ => (Health::NoData, format!("{pages_read} pages read, no price found")),
    }
}

pub fn classify(error: &FetchError) -> Health {
    match error {
        FetchError::Robots => Health::Blocked,
        FetchError::Status(status)
            if [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN, StatusCode::TOO_MANY_REQUESTS].contains(status) =>
        {
            Health::Blocked
        }
        _ => Health::Unreachable,
    }
}
