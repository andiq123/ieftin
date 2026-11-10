//! A per-client request limit for the search endpoint. A search makes this server
//! ask every live store, so one client must not be able to turn it into a flood.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

const WINDOW: Duration = Duration::from_secs(60);
/// Searches one client may make per window; generous for a person, tight for a script.
const MAX_PER_WINDOW: u32 = 60;
/// Past this many tracked clients, finished windows are dropped.
const PRUNE_ABOVE: usize = 10_000;
/// Set by Cloudflare to the visitor's address when the app is reached through its tunnel.
const FORWARDED_CLIENT: &str = "cf-connecting-ip";

/// When each client's current window started and how many requests it has made in it.
#[derive(Clone, Default)]
pub struct RateLimit(Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>);

impl RateLimit {
    /// Counts a request; `Err` holds the seconds until the client may try again.
    fn admit(&self, client: IpAddr, now: Instant) -> Result<(), u64> {
        let mut clients = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if clients.len() > PRUNE_ABOVE {
            clients.retain(|_, (started, _)| now.duration_since(*started) < WINDOW);
        }
        let (started, count) = clients.entry(client).or_insert((now, 0));
        if now.duration_since(*started) >= WINDOW {
            (*started, *count) = (now, 0);
        }
        if *count >= MAX_PER_WINDOW {
            return Err((WINDOW - now.duration_since(*started)).as_secs().max(1));
        }
        *count += 1;
        Ok(())
    }
}

pub async fn enforce(
    State(limit): State<RateLimit>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    match limit.admit(client(request.headers(), peer), Instant::now()) {
        Ok(()) => next.run(request).await,
        Err(seconds) => {
            (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, seconds.to_string())], "too many searches")
                .into_response()
        }
    }
}

/// The visitor's address. Behind the tunnel every connection comes from the same
/// local proxy, so the address it forwards is used when the peer is local.
fn client(headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    let forwarded = headers.get(FORWARDED_CLIENT).and_then(|value| value.to_str().ok()?.trim().parse().ok());
    forwarded.filter(|_| peer.ip().is_loopback()).unwrap_or_else(|| peer.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(203, 0, 113, 7));

    #[test]
    fn refuses_past_the_limit_until_the_window_ends() {
        let (limit, start) = (RateLimit::default(), Instant::now());
        for _ in 0..MAX_PER_WINDOW {
            assert_eq!(limit.admit(CLIENT, start), Ok(()));
        }
        assert_eq!(limit.admit(CLIENT, start + Duration::from_secs(20)), Err(40));
        assert_eq!(limit.admit(CLIENT, start + WINDOW), Ok(()));
    }

    #[test]
    fn trusts_the_forwarded_address_only_from_the_local_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert(FORWARDED_CLIENT, "203.0.113.7".parse().unwrap());
        let (local, remote) = ("127.0.0.1:5000".parse().unwrap(), "192.168.1.9:5000".parse::<SocketAddr>().unwrap());
        assert_eq!(client(&headers, local), CLIENT);
        assert_eq!(client(&headers, remote), remote.ip());
    }
}
