# Ieftin

Price comparison for the Moldovan market: search a product and see every store's
offer, ranked from cheapest to most expensive, with links to the real listings.

*Ieftin* is Romanian for "cheap".

## Run

```bash
./start.sh
```

To throw the index away and rebuild it from scratch (also needed after a change to
the database schema):

```bash
./start.sh --fresh
```

Requires Docker, Rust and Node.js. The script starts PostgreSQL, builds and starts the
backend (which applies migrations and starts the health checks and daily crawls), then starts the frontend at
<http://localhost:5180>. Settings live in `.env` (created from `.env.example`).

Live stores are searchable within seconds; indexed stores fill in as they are crawled.

## How it works

```
                 ┌─ live stores: their own search page, asked now ─┐
search ──────────┤                                                  ├──▶ one ranked list
                 └─ indexed stores: PostgreSQL, crawled daily ──────┘
```

Each store is read in one of two ways:

- **Live** – when its `robots.txt` allows automated use of its search page, the
  store is asked at search time. Prices are current, nothing is stored, and answers
  are cached in memory for 10 minutes. A store that has not answered within 7
  seconds is left out of that search and named on the page.
- **Indexed** – stores that forbid automated search are crawled once a day into
  PostgreSQL instead (robots.txt obeyed, one request at a time with a pause), and
  searched there. The index holds only current offers: whatever a crawl no longer
  finds is deleted when it finishes, and a store that fails its health check is
  removed from the index at once.

Both feed one pipeline:

- **Matching** – every word must appear in the title; case, diacritics and
  `256GB`/`256 GB` differences are ignored, and common synonyms are expanded (`ps5`
  finds "PlayStation 5"). The same rule is applied to the index and to live results.
- **Accessory floor** – "playstation 5" matches consoles and many games; sorted by
  price, the console would be buried. The backend finds where the product's prices
  start (from categories when offers have them, otherwise the widest gap in the
  prices) and hides what is below by default; the UI says so and offers "show all".
- **Health check** – once a day every store is tested (a test search for live
  stores, a short crawl for indexed ones). Only stores that pass are used. The
  *Stores* page shows each store's mode, state and crawl progress.
- **Price history** – every price change is kept (`price_history`): a daily crawl
  adds a point when an offer's price differs from the last one recorded, and a live
  search does the same for what it just read. `GET /api/history?store_id=&external_id=`
  returns the points; each result has a history button that opens an offer page charting them.
- **Saved offers** – kept in the browser's local storage, no account. The *Saved*
  page shows each one's current price and how it moved since it was saved.

## Layout

```
start.sh              starts database, backend, frontend; Ctrl+C stops everything
docker-compose.yml    PostgreSQL
backend/              Rust: axum, sqlx, reqwest, scraper
  migrations/         schema
  src/stores/         store adapters and the registry of all stores
  src/live.rs         live search across stores, with timeout and cache
  src/crawler/        fetcher (robots.txt, pacing), crawl, health checks, scheduler
  src/search/         text matching and the ranking pipeline (pure, unit-tested)
  src/repo.rs         all SQL
  src/api/            HTTP handlers
frontend/             React 19, TypeScript, Vite, TanStack Query
  src/api/            typed client and query hooks
  src/lib/            URL-backed state, routing and transitions, sorting, formatting
  src/components/     UI
  src/styles/         design tokens, motion, one stylesheet per area
  public/             web app manifest and icons (installable as a PWA)
```

## Stores

All stores are listed in `backend/src/stores/registry.rs`; the *Stores* page shows
which ones currently pass their health check.

A live store whose search results are plain HTML cards is one registry entry of CSS
selectors:

```rust
CssSearch {
    id: "bomba",
    name: "Bomba",
    home: "https://bomba.md",
    search: "https://bomba.md/ro/cautare/?query={}",
    probe: "samsung",
    card: ".card-product-item",
    link: ".product-info a",
    title: None,
    price: ".product-price .price",
    old_price: Some(".product-price .old-price"),
    image: Some((".product-photo img", "src")),
},
```

An indexed store that publishes schema.org product data is one line naming its
sitemap and which entries are product pages:

```rust
Structured::new("alo", "Alo", "https://alo.md", "https://alo.md/sitemap.xml")
    .sitemaps(&["category-products/"])
    .pages(&["/ro/product/"]),
```

Anything else gets a small adapter implementing `LiveStore` or `Store`, as
`darwin.rs` and `maximum.rs` do.

## API

- `GET /api/search?q=…&store=&category=&min_price=&max_price=&in_stock=&accessories=&sort=price_asc|price_desc&limit=&offset=`
- `GET /api/stores` – every store with its health, index size, live activity
  (checking, or crawl progress with an estimate) and next scheduled run
- `GET /api/health`

## Development

```bash
cd backend && cargo test && cargo clippy
```

```bash
cd frontend && npm run lint && npm run build
```
