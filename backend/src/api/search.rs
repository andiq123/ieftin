use std::{net::SocketAddr, time::Instant};

use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use super::{AppState, limit};
use crate::{
    domain::Offer,
    error::ApiError,
    live::SourceReport,
    search::{Filters, Results, Sort, TextQuery, assemble, normalize},
};

const DEFAULT_LIMIT: usize = 48;
const MAX_LIMIT: usize = 100;
/// Longer than any product name; anything beyond is not a search.
const MAX_QUERY_CHARS: usize = 120;
const BANI_PER_LEU: i64 = 100;

/// Query string of `GET /api/search`; prices are in lei.
#[derive(Debug, Deserialize)]
pub struct Params {
    q: String,
    store: Option<String>,
    category: Option<String>,
    min_price: Option<i64>,
    max_price: Option<i64>,
    #[serde(default)]
    in_stock: bool,
    #[serde(default)]
    accessories: bool,
    #[serde(default)]
    sort: Sort,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl From<&Params> for Filters {
    fn from(params: &Params) -> Self {
        let to_bani = |lei: i64| lei.clamp(0, i64::MAX / BANI_PER_LEU) * BANI_PER_LEU;
        Self {
            store: params.store.clone().filter(|store| !store.is_empty()),
            category: params.category.clone().filter(|category| !category.is_empty()),
            min_price: params.min_price.map(to_bani),
            max_price: params.max_price.map(to_bani),
            in_stock_only: params.in_stock,
            include_accessories: params.accessories,
            sort: params.sort,
            limit: params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
            offset: params.offset.unwrap_or(0),
        }
    }
}

#[derive(Serialize)]
pub struct Response {
    #[serde(flatten)]
    results: Results,
    /// How each store answered this search.
    sources: Vec<SourceReport>,
}

/// Asks every working store at once and ranks what has arrived so far. Stores still
/// answering are reported as pending; asking again includes them as they come in.
pub async fn search(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<Params>,
) -> Result<Json<Response>, ApiError> {
    if params.q.chars().count() > MAX_QUERY_CHARS {
        return Err(ApiError::BadRequest("q is too long"));
    }
    let text = TextQuery::parse(&params.q).ok_or(ApiError::BadRequest("q must contain a word"))?;
    let key = text.normalized();
    let working = |store: &str| state.health.of(store).usable();
    // Only a search that sends requests to stores counts against the client's limit.
    if state.live.is_new(working, &key) {
        let client = limit::client(&headers, peer);
        state
            .limit
            .admit(client, Instant::now())
            .map_err(|retry_after_seconds| ApiError::TooManyRequests { retry_after_seconds })?;
    }
    let answers = state.live.search(working, params.q.trim(), &key).await;

    let mut offers = Vec::new();
    let mut sources = Vec::with_capacity(answers.len());
    for (report, found) in answers {
        // A store's own search is fuzzier than ours; keep only what really matches.
        let matching = found.iter().filter(|offer| text.matches(&normalize(&offer.title)));
        offers.extend(matching.map(|offer| Offer::new(report.store_id, offer)));
        sources.push(report);
    }
    Ok(Json(Response { results: assemble(offers, &text, &Filters::from(&params)), sources }))
}
