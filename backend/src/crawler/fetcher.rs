use std::{collections::HashMap, io::Read, sync::Arc, time::Duration};

use flate2::read::GzDecoder;
use reqwest::{Client, StatusCode};
use tokio::{sync::Mutex, time::Instant};
use url::{Position, Url};

use super::robots::Robots;

const AGENT: &str = "IeftinBot";
/// Identifies the crawler honestly so stores can recognise (and rate limit) it.
const USER_AGENT: &str = concat!("IeftinBot/", env!("CARGO_PKG_VERSION"), " (price comparison)");
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
/// A store may ask for a slower pace than ours, within reason.
const MAX_CRAWL_DELAY: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP {}", .0.as_u16())]
    Status(StatusCode),
    #[error("disallowed by robots.txt")]
    Robots,
    #[error("connection failed")]
    Network(#[from] reqwest::Error),
    #[error("unreadable response: {0}")]
    Body(String),
}

impl FetchError {
    fn is_transient(&self) -> bool {
        match self {
            Self::Status(status) => status.is_server_error() || *status == StatusCode::TOO_MANY_REQUESTS,
            Self::Network(_) => true,
            Self::Robots | Self::Body(_) => false,
        }
    }
}

/// A polite HTTP client, one per store: it obeys robots.txt, spaces requests out
/// and retries transient failures with backoff.
pub struct Fetcher {
    client: Client,
    delay: Duration,
    /// Tries per request; transient failures are retried with backoff.
    attempts: u32,
    next_slot: Mutex<Instant>,
    robots: Mutex<HashMap<String, Arc<Robots>>>,
}

impl Fetcher {
    pub fn new(delay: Duration, attempts: u32) -> reqwest::Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client, delay, attempts, next_slot: Mutex::new(Instant::now()), robots: Mutex::default() })
    }

    /// Fetches a document as text.
    pub async fn get(&self, url: &Url) -> Result<String, FetchError> {
        let robots = self.robots_for(url).await;
        if !robots.allows(&url[Position::BeforePath..Position::AfterQuery]) {
            return Err(FetchError::Robots);
        }
        let delay = robots.crawl_delay.map_or(self.delay, |asked| asked.clamp(self.delay, MAX_CRAWL_DELAY));
        self.get_with_retries(url, delay).await
    }

    /// The robots.txt rules of the URL's host, fetched once. A missing or
    /// unreadable file means no restrictions.
    async fn robots_for(&self, url: &Url) -> Arc<Robots> {
        let origin = url[..Position::BeforePath].to_owned();
        if let Some(robots) = self.robots.lock().await.get(&origin) {
            return robots.clone();
        }
        let robots = match Url::parse(&format!("{origin}/robots.txt")) {
            Ok(location) => self.get_with_retries(&location, self.delay).await.unwrap_or_default(),
            Err(_) => String::new(),
        };
        let robots = Arc::new(Robots::parse(&robots, AGENT));
        self.robots.lock().await.insert(origin, robots.clone());
        robots
    }

    async fn get_with_retries(&self, url: &Url, delay: Duration) -> Result<String, FetchError> {
        let mut attempt = 1;
        loop {
            self.wait_for_slot(delay).await;
            match self.try_get(url).await {
                Err(error) if error.is_transient() && attempt < self.attempts => {
                    tracing::debug!(%url, attempt, "retrying: {error}");
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    attempt += 1;
                }
                result => return result,
            }
        }
    }

    async fn wait_for_slot(&self, delay: Duration) {
        let slot = {
            let mut next_slot = self.next_slot.lock().await;
            let slot = (*next_slot).max(Instant::now());
            *next_slot = slot + delay;
            slot
        };
        tokio::time::sleep_until(slot).await;
    }

    async fn try_get(&self, url: &Url) -> Result<String, FetchError> {
        let response = self.client.get(url.clone()).send().await?;
        if !response.status().is_success() {
            return Err(FetchError::Status(response.status()));
        }
        decode(&response.bytes().await?)
    }
}

/// Decodes a body as UTF-8, transparently inflating `.gz` files such as sitemaps.
fn decode(bytes: &[u8]) -> Result<String, FetchError> {
    if bytes.starts_with(&GZIP_MAGIC) {
        let mut text = String::new();
        return match GzDecoder::new(bytes).read_to_string(&mut text) {
            Ok(_) => Ok(text),
            Err(error) => Err(FetchError::Body(error.to_string())),
        };
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}
