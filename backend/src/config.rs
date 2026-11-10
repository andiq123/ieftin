use std::{env, net::SocketAddr, str::FromStr, time::Duration};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub crawl: CrawlConfig,
}

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub enabled: bool,
    /// How long a finished crawl stays fresh before the store is crawled again.
    pub interval: Duration,
    /// Minimum pause between two requests to the same store.
    pub delay: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
            bind_addr: var_or("BIND_ADDR", "127.0.0.1:8090".parse()?)?,
            crawl: CrawlConfig {
                enabled: var_or("CRAWL_ENABLED", true)?,
                interval: Duration::from_secs(3600 * var_or("CRAWL_INTERVAL_HOURS", 24)?),
                delay: Duration::from_millis(var_or("CRAWL_DELAY_MS", 300)?),
            },
        })
    }
}

fn var_or<T>(key: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(key) {
        Ok(raw) => raw.trim().parse().with_context(|| format!("invalid {key}: {raw:?}")),
        Err(_) => Ok(default),
    }
}
