/** Mirrors the backend's JSON. Money is integer bani (1 leu = 100 bani). */

export interface Offer {
  store_id: string
  /** With `store_id`, identifies the offer and its price history. */
  external_id: string
  title: string
  url: string
  image_url: string | null
  category: string | null
  price_bani: number
  old_price_bani: number | null
  in_stock: boolean | null
  /** Read from the store just now, rather than from the daily index. */
  live: boolean
}

/** A price an offer changed to, and when that was first seen. */
export interface PricePoint {
  price_bani: number
  observed_at: string
}

export interface PriceHistory {
  /** The offer as indexed now; null for a live store's offer or one no longer sold. */
  offer: Offer | null
  /** Every price change seen, oldest first. */
  points: PricePoint[]
}

/** How one live store answered a search. */
export interface SourceReport {
  store_id: string
  ok: boolean
  elapsed_ms: number
}

export interface Facet {
  key: string
  count: number
  min_price_bani: number
}

export interface SearchResults {
  total: number
  offers: Offer[]
  stores: Facet[]
  categories: Facet[]
  accessory_floor_bani: number | null
  sources: SourceReport[]
}

export interface Store {
  id: string
  name: string
  url: string
  /** `live`: searched at query time. `indexed`: crawled daily. */
  mode: 'live' | 'indexed'
  offers: number
  /** Result of the latest daily check; null until the first one has run. */
  health_status: Health | null
  health_detail: string | null
  health_checked_at: string | null
  last_crawl_started_at: string | null
  last_crawl_finished_at: string | null
  /** What the store's background task is doing right now. */
  activity: Activity
  /** Estimated seconds until the running crawl finishes. */
  eta_seconds: number | null
  next_check_at: string | null
  next_crawl_at: string | null
}

export type Activity =
  | { state: 'idle' | 'checking' }
  | { state: 'crawling'; started_at: string; pages_done: number; pages_pending: number; offers: number }

export type Health = 'ok' | 'blocked' | 'unreachable' | 'no_data'

export type Sort = 'price_asc' | 'price_desc'
