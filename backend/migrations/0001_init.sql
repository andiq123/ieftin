-- Every known store, with the result of its latest daily health check.
create table stores (
    id                     text primary key,
    name                   text not null,
    url                    text not null,
    -- 'live': asked at search time. 'indexed': crawled daily into `offers`.
    mode                   text not null check (mode in ('live', 'indexed')),
    health_status          text,
    health_detail          text,
    health_checked_at      timestamptz,
    last_crawl_started_at  timestamptz,
    last_crawl_finished_at timestamptz
);

-- The index: current offers of healthy indexed stores, and nothing else.
-- Live stores are never stored; offers a crawl no longer finds are deleted.
create table offers (
    store_id       text not null references stores (id) on delete cascade,
    -- The store's own stable product id.
    external_id    text not null,
    title          text not null,
    url            text not null,
    image_url      text,
    category       text,
    price_bani     bigint not null check (price_bani > 0),
    old_price_bani bigint,
    in_stock       boolean,
    -- Lower-cased, diacritic-free text produced by the backend (see search::normalize).
    search_text    text not null,
    search         tsvector generated always as (to_tsvector('simple', search_text)) stored,
    -- Set by every crawl that finds the offer; older rows are pruned when a crawl ends.
    last_seen_at   timestamptz not null default now(),
    primary key (store_id, external_id)
);

create index offers_search_idx on offers using gin (search);
