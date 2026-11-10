use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::{
    domain::Offer,
    error::{AppError, AppResult},
    live::SourceReport,
    repo,
    search::{Filters, Results, Sort, TextQuery, assemble, normalize},
};

const DEFAULT_LIMIT: usize = 48;
const MAX_LIMIT: usize = 100;
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
    /// How each live store answered this search.
    sources: Vec<SourceReport>,
}

/// Asks the live stores and the index at the same time, then ranks everything together.
pub async fn search(State(state): State<AppState>, Query(params): Query<Params>) -> AppResult<Json<Response>> {
    let text = TextQuery::parse(&params.q).ok_or(AppError::BadRequest("q must contain a word"))?;
    let live_stores = repo::healthy_live_stores(&state.pool).await?;
    let (expression, cache_key) = (text.expression(), text.normalized());
    let (indexed, live) = tokio::join!(
        repo::matching_offers(&state.pool, &expression),
        state.live.search(&live_stores, params.q.trim(), &cache_key),
    );

    let mut offers = indexed?;
    let mut sources = Vec::with_capacity(live.len());
    for (report, found) in live {
        // A store's own search is fuzzier than ours; keep only what really matches.
        let matching = found.iter().filter(|offer| text.matches(&normalize(&offer.title)));
        offers.extend(matching.map(|offer| Offer::live(report.store_id, offer)));
        if !found.is_empty() {
            // Off the request path: the answer does not wait for the history to be written.
            let (pool, store) = (state.pool.clone(), report.store_id);
            tokio::spawn(async move {
                if let Err(error) = repo::record_prices(&pool, store, &found).await {
                    tracing::warn!(store, "recording live prices failed: {error}");
                }
            });
        }
        sources.push(report);
    }
    Ok(Json(Response { results: assemble(offers, &text, &Filters::from(&params)), sources }))
}
