//! All SQL lives here; the rest of the backend never touches the schema.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{
    domain::{Health, Offer, PricePoint, ScrapedOffer, StoreOverview},
    search::normalize,
    stores::Registry,
};

/// The most index rows one search looks at; far more than any real query matches.
const MAX_MATCHES: i64 = 5_000;

/// The columns of an indexed offer, as `Offer` reads them.
macro_rules! offer_columns {
    () => {
        "store_id, external_id, title, url, image_url, category, price_bani, old_price_bani, in_stock"
    };
}

/// Makes the `stores` table mirror the compiled-in registry: adds and updates
/// stores (keeping their health and crawl state), removes ones no longer known, and
/// empties the index of any store that is now searched live.
pub async fn sync_stores(pool: &PgPool, registry: &Registry) -> sqlx::Result<()> {
    let mut transaction = pool.begin().await?;
    for (store, live) in registry.stores() {
        sqlx::query(
            "insert into stores (id, name, url, mode) values ($1, $2, $3, $4)
             on conflict (id) do update set name = excluded.name, url = excluded.url, mode = excluded.mode",
        )
        .bind(store.id())
        .bind(store.name())
        .bind(store.home())
        .bind(if live { "live" } else { "indexed" })
        .execute(&mut *transaction)
        .await?;
    }
    let known: Vec<&str> = registry.stores().map(|(store, _)| store.id()).collect();
    sqlx::query("delete from stores where id <> all($1)").bind(known).execute(&mut *transaction).await?;
    sqlx::query("delete from offers where store_id in (select id from stores where mode = 'live')")
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await
}

/// Live stores whose last health check passed: the ones worth asking at search time.
pub async fn healthy_live_stores(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar("select id from stores where mode = 'live' and health_status = 'ok'").fetch_all(pool).await
}

/// Every indexed offer whose title matches the `to_tsquery` expression.
pub async fn matching_offers(pool: &PgPool, expression: &str) -> sqlx::Result<Vec<Offer>> {
    sqlx::query_as(concat!(
        "select ",
        offer_columns!(),
        " from offers where search @@ to_tsquery('simple', $1) limit $2"
    ))
    .bind(expression)
    .bind(MAX_MATCHES)
    .fetch_all(pool)
    .await
}

/// One indexed offer; `None` for offers of live stores, which are never stored.
pub async fn offer(pool: &PgPool, store: &str, external_id: &str) -> sqlx::Result<Option<Offer>> {
    sqlx::query_as(concat!("select ", offer_columns!(), " from offers where store_id = $1 and external_id = $2"))
        .bind(store)
        .bind(external_id)
        .fetch_optional(pool)
        .await
}

/// Every price change recorded for an offer, oldest first.
pub async fn price_history(pool: &PgPool, store: &str, external_id: &str) -> sqlx::Result<Vec<PricePoint>> {
    sqlx::query_as(
        "select price_bani, observed_at from price_history
         where store_id = $1 and external_id = $2 order by observed_at",
    )
    .bind(store)
    .bind(external_id)
    .fetch_all(pool)
    .await
}

pub async fn stores_overview(pool: &PgPool) -> sqlx::Result<Vec<StoreOverview>> {
    sqlx::query_as(
        "select s.id, s.name, s.url, s.mode, s.health_status, s.health_detail, s.health_checked_at,
                s.last_crawl_started_at, s.last_crawl_finished_at, count(o.store_id) as offers
         from stores s left join offers o on o.store_id = s.id
         group by s.id order by s.name",
    )
    .fetch_all(pool)
    .await
}

/// What the scheduler needs to decide a store's next step.
#[derive(sqlx::FromRow)]
pub struct CrawlState {
    pub health_status: Option<Health>,
    pub health_checked_at: Option<DateTime<Utc>>,
    pub last_crawl_finished_at: Option<DateTime<Utc>>,
}

pub async fn crawl_state(pool: &PgPool, store: &str) -> sqlx::Result<CrawlState> {
    sqlx::query_as("select health_status, health_checked_at, last_crawl_finished_at from stores where id = $1")
        .bind(store)
        .fetch_one(pool)
        .await
}

/// Records a health check. A store that fails it leaves the index at once, so
/// search never shows offers from a store that cannot be read.
pub async fn save_health(pool: &PgPool, store: &str, status: Health, detail: &str) -> sqlx::Result<()> {
    let mut transaction = pool.begin().await?;
    sqlx::query("update stores set health_status = $2, health_detail = $3, health_checked_at = now() where id = $1")
        .bind(store)
        .bind(status)
        .bind(detail)
        .execute(&mut *transaction)
        .await?;
    if status != Health::Ok {
        sqlx::query("delete from offers where store_id = $1").bind(store).execute(&mut *transaction).await?;
    }
    transaction.commit().await
}

pub async fn mark_crawl_started(pool: &PgPool, store: &str) -> sqlx::Result<()> {
    sqlx::query("update stores set last_crawl_started_at = now() where id = $1")
        .bind(store)
        .execute(pool)
        .await
        .map(drop)
}

/// Completes a crawl: offers it did not find are no longer sold and leave the index.
pub async fn mark_crawl_finished(pool: &PgPool, store: &str) -> sqlx::Result<u64> {
    let mut transaction = pool.begin().await?;
    let pruned = sqlx::query(
        "delete from offers o using stores s
         where o.store_id = $1 and s.id = o.store_id and o.last_seen_at < s.last_crawl_started_at",
    )
    .bind(store)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    sqlx::query("update stores set last_crawl_finished_at = now() where id = $1")
        .bind(store)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(pruned)
}

/// A page may repeat a product; one statement cannot write the same row twice.
fn distinct(offers: &[ScrapedOffer]) -> Vec<&ScrapedOffer> {
    let mut seen = HashSet::new();
    offers.iter().filter(|offer| seen.insert(&offer.external_id)).collect()
}

/// Adds each offer's price to its history, unless it is the price recorded last.
async fn record_changes(connection: &mut PgConnection, store: &str, offers: &[&ScrapedOffer]) -> sqlx::Result<()> {
    sqlx::query(
        "insert into price_history (store_id, external_id, price_bani)
         select $1, seen.external_id, seen.price_bani
         from unnest($2::text[], $3::bigint[]) as seen (external_id, price_bani)
         where seen.price_bani is distinct from (
             select price_bani from price_history
             where store_id = $1 and external_id = seen.external_id
             order by observed_at desc limit 1)
         on conflict do nothing",
    )
    .bind(store)
    .bind(offers.iter().map(|offer| offer.external_id.as_str()).collect::<Vec<_>>())
    .bind(offers.iter().map(|offer| offer.price).collect::<Vec<_>>())
    .execute(connection)
    .await
    .map(drop)
}

/// Records the prices a live search just read, so live stores get a history too.
pub async fn record_prices(pool: &PgPool, store: &str, offers: &[ScrapedOffer]) -> sqlx::Result<()> {
    record_changes(&mut *pool.acquire().await?, store, &distinct(offers)).await
}

/// Inserts or refreshes a page of offers, and notes any price that changed.
pub async fn upsert_offers(pool: &PgPool, store: &str, offers: &[ScrapedOffer]) -> sqlx::Result<usize> {
    let offers = distinct(offers);
    if offers.is_empty() {
        return Ok(0);
    }
    let mut transaction = pool.begin().await?;
    let column = |value: fn(&ScrapedOffer) -> Option<String>| -> Vec<Option<String>> {
        offers.iter().map(|offer| value(offer)).collect()
    };

    sqlx::query(
        "insert into offers (store_id, external_id, title, url, image_url, category,
                             search_text, price_bani, old_price_bani, in_stock)
         select $1, * from unnest($2::text[], $3::text[], $4::text[], $5::text[], $6::text[],
                                  $7::text[], $8::bigint[], $9::bigint[], $10::bool[])
         on conflict (store_id, external_id) do update set
             title = excluded.title, url = excluded.url, image_url = excluded.image_url,
             category = excluded.category, search_text = excluded.search_text,
             price_bani = excluded.price_bani, old_price_bani = excluded.old_price_bani,
             in_stock = excluded.in_stock, last_seen_at = now()",
    )
    .bind(store)
    .bind(column(|offer| Some(offer.external_id.clone())))
    .bind(column(|offer| Some(offer.title.clone())))
    .bind(column(|offer| Some(offer.url.to_string())))
    .bind(column(|offer| offer.image_url.as_ref().map(ToString::to_string)))
    .bind(column(|offer| offer.category.clone()))
    .bind(column(|offer| {
        Some(normalize(&format!("{} {}", offer.title, offer.category.as_deref().unwrap_or_default())))
    }))
    .bind(offers.iter().map(|offer| offer.price).collect::<Vec<_>>())
    .bind(offers.iter().map(|offer| offer.old_price).collect::<Vec<_>>())
    .bind(offers.iter().map(|offer| offer.in_stock).collect::<Vec<_>>())
    .execute(&mut *transaction)
    .await?;
    record_changes(&mut transaction, store, &offers).await?;
    transaction.commit().await?;
    Ok(offers.len())
}
