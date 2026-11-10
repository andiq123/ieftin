/** Mirrors the backend's JSON. Money is integer bani (1 leu = 100 bani). */

export interface Offer {
  store_id: string
  /** With `store_id`, identifies the offer, e.g. among the saved ones. */
  external_id: string
  title: string
  url: string
  image_url: string | null
  category: string | null
  price_bani: number
  old_price_bani: number | null
  in_stock: boolean | null
}

/** How one store answered a search. */
export interface SourceReport {
  store_id: string
  ok: boolean
  /** Still being read by the backend: asking again soon will include it. */
  pending: boolean
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
  /** Matches that are accessories (cases, cables, games…) rather than the product. */
  accessories_total: number
  /** How many of those this answer leaves out; none when only accessories matched. */
  accessories_hidden: number
  sources: SourceReport[]
}

export interface Store {
  id: string
  name: string
  url: string
  /** Result of the latest test search; null until the first one has finished. */
  status: Health | null
  detail: string | null
  checked_at: string | null
  /** How long the store took to answer its latest test, when it answered. */
  latency_ms: number | null
  /** A test search is running right now. */
  checking: boolean
  next_check_at: string | null
}

export type Health = 'ok' | 'blocked' | 'unreachable' | 'no_data'

export type Sort = 'price_asc' | 'price_desc'
