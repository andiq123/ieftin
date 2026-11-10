use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::{
    domain::{Offer, PricePoint},
    error::AppResult,
    repo,
};

/// Query string of `GET /api/history`: the two fields that identify an offer.
#[derive(Debug, Deserialize)]
pub struct Params {
    store_id: String,
    external_id: String,
}

#[derive(Serialize)]
pub struct Response {
    /// The offer as indexed now; `None` for a live store's offer or one no longer sold.
    offer: Option<Offer>,
    /// Every price change seen, oldest first.
    points: Vec<PricePoint>,
}

pub async fn history(State(state): State<AppState>, Query(params): Query<Params>) -> AppResult<Json<Response>> {
    let (offer, points) = tokio::try_join!(
        repo::offer(&state.pool, &params.store_id, &params.external_id),
        repo::price_history(&state.pool, &params.store_id, &params.external_id),
    )?;
    Ok(Json(Response { offer, points }))
}
