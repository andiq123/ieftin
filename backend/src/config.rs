use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::{Context, Result};
use axum::http::HeaderValue;

/// Everything is optional: with no settings at all the server runs for local development.
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    /// The websites whose pages may call the API; `None` allows any.
    pub cors_origins: Option<Vec<HeaderValue>>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self { bind_addr: bind_addr()?, cors_origins: cors_origins()? })
    }
}

/// `BIND_ADDR` when set. Otherwise a hosting platform's `PORT`, on every interface
/// because the platform routes to it from outside; otherwise local development.
fn bind_addr() -> Result<SocketAddr> {
    if let Ok(addr) = env::var("BIND_ADDR") {
        return addr.trim().parse().with_context(|| format!("invalid BIND_ADDR: {addr:?}"));
    }
    Ok(match env::var("PORT") {
        Ok(port) => {
            let port = port.trim().parse().with_context(|| format!("invalid PORT: {port:?}"))?;
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))
        }
        Err(_) => SocketAddr::from((Ipv4Addr::LOCALHOST, 8090)),
    })
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
