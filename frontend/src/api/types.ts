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
  /** Result of the latest test search; null until the first one has finished. */
  status: Health | null
  detail: string | null
  checked_at: string | null
  /** A test search is running right now. */
  checking: boolean
  next_check_at: string | null
}

export type Health = 'ok' | 'blocked' | 'unreachable' | 'no_data'

export type Sort = 'price_asc' | 'price_desc'
