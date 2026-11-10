# Ieftin

Price comparison for the Moldovan market: search a product and see every store's
offer, ranked from cheapest to most expensive, with links to the real listings.
Every price shown was read from the store at the moment of the search.

*Ieftin* is Romanian for "cheap".

## Run

```bash
./start.sh
```

Requires Rust and Node.js. The script builds and starts the backend, then starts the
frontend at <http://localhost:5180>. There is no database and nothing else to set up.

## How it works

```
search ──▶ every working store's own search page, asked now ──▶ one ranked list
```

- **Live search** – a search asks each store's own search page at that moment
  (robots.txt obeyed, requests to one store spaced out). It answers with what has
  arrived after about a second; the other stores keep going in the background for up
  to 25 seconds, and the page keeps asking, so results appear store by store with the
  stores still loading named. Answers are reused for 10 minutes, so paging and
  repeated searches cost the stores nothing.
- **Matching** – every word must appear in the title; case, diacritics and
  `256GB`/`256 GB` differences are ignored, and common synonyms are expanded (`ps5`
  finds "PlayStation 5"). Stores' own searches are fuzzier, so this is applied to
  what they return.
- **Product first** – "iphone 15 pro" matches two phones and a hundred cases; sorted
  by price, the phones would be buried. Listings that name an accessory (husă, case,
  sticlă, cablu…) are set aside, and so is whatever is priced far below where the
  product's own prices start. The page says how many were hidden and offers "show
  all". When nothing but accessories matched, the page says the product was not
  found and offers them. They are the product when the search asks for one
  ("husa iphone 15").
- **Health check** – each store is tested with a known query when the server starts
  and every six hours after. One that fails is left out of searches and retried
  every half hour. The *Stores* page shows each store's state. Nothing is stored: a
  restart simply tests again. The page follows the tests as they run and shows how
  fast each store answered.
- **Saved offers** – kept in the browser's local storage, no account, as they were
  when saved.

## Settings

Each side has its own file, created from its example on first run. Nothing is
required: neither file needs editing to develop.

| File | Setting | Default |
| --- | --- | --- |
| `backend/.env` | `BIND_ADDR` | the platform's `PORT` on every interface, else `127.0.0.1:8090` |
| | `CORS_ORIGINS` | any website may call the API (it is public, read-only, cookie-free) |
| `frontend/.env` | `VITE_API_URL` | empty: the backend is at the page's own address |
| | `FRONTEND_PORT`, `BACKEND_URL` (development only) | `5180`, `http://127.0.0.1:8090` |

`start.sh` passes the backend's address to the dev server, so the two cannot drift.

## Deploy

The two sides deploy separately.

- **Backend** – anywhere that builds a Rust binary. It needs no settings and no
  database: it listens on the platform's `PORT` and stops cleanly on SIGTERM.
  `/api/health` answers `ok`.
- **Frontend** – any static host. On Vercel, import the repository with `frontend` as
  the root directory (`frontend/vercel.json` holds the rest) and set `VITE_API_URL`
  to the backend's public address.

To restrict the API to your own frontend, set `CORS_ORIGINS` on the backend to its
address. New searches are limited to 60 a minute per visitor either way; repeats of a
search that is cached or still loading are free.

## Layout

```
start.sh              starts backend and frontend; Ctrl+C stops both
backend/              Rust: axum, reqwest, scraper
  src/stores/         store adapters and the list of all stores
  src/fetch/          polite HTTP client: robots.txt, pacing
  src/live.rs         searching every store at once, with timeout and cache
  src/health.rs       which stores work right now
  src/search/         text matching and the ranking pipeline (pure, unit-tested)
  src/api/            HTTP handlers and the search rate limit
frontend/             React 19, TypeScript, Vite, TanStack Query
  src/api/            typed client and query hooks
  src/lib/            URL-backed state, routing and transitions, formatting
  src/components/     UI
  src/styles/         design tokens, motion, one stylesheet per area
  public/             web app manifest, icons and the service worker that keeps the
                      app's own files for offline starts (installable as a PWA)
  icons/              icon sources; the PNGs in public/ are rendered from these
```

## Stores

All stores are listed in `backend/src/stores/registry.rs`; the *Stores* page shows
which ones currently pass their health check.

A store whose search results are plain HTML cards is one entry of CSS selectors:

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

Anything else gets a small adapter implementing `Store`, as `darwin.rs` does. A store
can only be added if its robots.txt allows automated use of its search page and its
results are in the page's HTML. That rules out, today: Maximum, Ultra, Enter, Atehno,
Breezy, TopSale (search disallowed) and Alo, BigShop, Xiaomi Store (results built in
the browser).

## API

- `GET /api/search?q=…&store=&category=&min_price=&max_price=&in_stock=&accessories=&sort=price_asc|price_desc&limit=&offset=`
- `GET /api/stores` – every store with the result of its latest test and when the
  next one is due
- `GET /api/health`

## Development

```bash
cd backend && cargo test && cargo clippy
```

```bash
cd frontend && npm run lint && npm run build
```
