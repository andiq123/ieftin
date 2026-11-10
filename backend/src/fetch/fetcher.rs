use std::{collections::HashMap, sync::Arc, time::Duration};

use reqwest::{Client, StatusCode};
use tokio::{sync::Mutex, time::Instant};
use url::{Position, Url};

use super::robots::Robots;

const AGENT: &str = "IeftinBot";
/// Identifies the bot honestly so stores can recognise (and rate limit) it.
const USER_AGENT: &str = concat!("IeftinBot/", env!("CARGO_PKG_VERSION"), " (price comparison)");
/// A store may ask for a slower pace than ours, within reason.
const MAX_PAUSE: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP {}", .0.as_u16())]
    Status(StatusCode),
    #[error("disallowed by robots.txt")]
    Robots,
    #[error("connection failed")]
    Network(#[from] reqwest::Error),
}

/// A polite HTTP client, one per store: it obeys robots.txt and spaces requests out.
pub struct Fetcher {
    client: Client,
    /// Minimum time between two requests to the store.
    pause: Duration,
    next_slot: Mutex<Instant>,
    robots: Mutex<HashMap<String, Arc<Robots>>>,
}

impl Fetcher {
    pub fn new(pause: Duration) -> reqwest::Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client, pause, next_slot: Mutex::new(Instant::now()), robots: Mutex::default() })
    }

    /// Fetches a page as text.
    pub async fn get(&self, url: &Url) -> Result<String, FetchError> {
        let robots = self.robots_for(url).await;
        if !robots.allows(&url[Position::BeforePath..Position::AfterQuery]) {
            return Err(FetchError::Robots);
        }
        let pause = robots.crawl_delay.map_or(self.pause, |asked| asked.clamp(self.pause, MAX_PAUSE));
        self.get_paced(url, pause).await
    }

    /// The robots.txt rules of the URL's host, fetched once. A missing or
    /// unreadable file means no restrictions.
    async fn robots_for(&self, url: &Url) -> Arc<Robots> {
        let origin = url[..Position::BeforePath].to_owned();
        if let Some(robots) = self.robots.lock().await.get(&origin) {
            return robots.clone();
        }
        let robots = match Url::parse(&format!("{origin}/robots.txt")) {
            Ok(location) => self.get_paced(&location, self.pause).await.unwrap_or_default(),
            Err(_) => String::new(),
        };
        let robots = Arc::new(Robots::parse(&robots, AGENT));
        self.robots.lock().await.insert(origin, robots.clone());
        robots
    }

    async fn get_paced(&self, url: &Url, pause: Duration) -> Result<String, FetchError> {
        let slot = {
            let mut next_slot = self.next_slot.lock().await;
            let slot = (*next_slot).max(Instant::now());
            *next_slot = slot + pause;
            slot
        };
        tokio::time::sleep_until(slot).await;

        let response = self.client.get(url.clone()).send().await?;
        if !response.status().is_success() {
            return Err(FetchError::Status(response.status()));
        }
        Ok(response.text().await?)
    }
}
