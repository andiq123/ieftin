-- Every price an offer has had: one row per change, so a price that stays put
-- costs nothing. Kept after an offer leaves the index, and also fed by live searches.
create table price_history (
    store_id    text not null references stores (id) on delete cascade,
    external_id text not null,
    price_bani  bigint not null check (price_bani > 0),
    observed_at timestamptz not null default now(),
    primary key (store_id, external_id, observed_at)
);

-- History starts with what the index holds today.
insert into price_history (store_id, external_id, price_bani, observed_at)
select store_id, external_id, price_bani, last_seen_at from offers;
