use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use anyhow::{Context, Result};
use axum::http::HeaderValue;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    /// The websites whose pages may call the API; `None` allows any.
    pub cors_origins: Option<Vec<HeaderValue>>,
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
            bind_addr: bind_addr()?,
            cors_origins: cors_origins()?,
            crawl: CrawlConfig {
                enabled: var_or("CRAWL_ENABLED", true)?,
                interval: Duration::from_secs(3600 * var_or("CRAWL_INTERVAL_HOURS", 24)?),
                delay: Duration::from_millis(var_or("CRAWL_DELAY_MS", 300)?),
            },
        })
    }
}

/// `BIND_ADDR` when set. Otherwise a hosting platform's `PORT`, on every interface
/// because the platform routes to it from outside; otherwise local development.
fn bind_addr() -> Result<SocketAddr> {
    let fallback = match env::var("PORT") {
        Ok(port) => {
            let port = port.trim().parse().with_context(|| format!("invalid PORT: {port:?}"))?;
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))
        }
        Err(_) => SocketAddr::from((Ipv4Addr::LOCALHOST, 8090)),
    };
    var_or("BIND_ADDR", fallback)
}

/// `CORS_ORIGINS`: comma-separated origins such as `https://ieftin.vercel.app`.
/// Unset or empty allows every website: the API is public, read-only and cookie-free.
fn cors_origins() -> Result<Option<Vec<HeaderValue>>> {
    let raw = env::var("CORS_ORIGINS").unwrap_or_default();
    let origins = raw.split(',').map(|origin| origin.trim().trim_end_matches('/')).filter(|origin| !origin.is_empty());
    let origins: Vec<HeaderValue> = origins
        .map(|origin| origin.parse().with_context(|| format!("invalid origin in CORS_ORIGINS: {origin:?}")))
        .collect::<Result<_>>()?;
    Ok(Some(origins).filter(|origins| !origins.is_empty()))
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
