import type { Offer } from '../api/types.ts'

export const ROUTES = { search: '/', saved: '/saved', stores: '/stores', offer: '/offer' } as const

/** The store and product id that identify an offer and its price history. */
export type OfferKey = Pick<Offer, 'store_id' | 'external_id'>

/** Keeps the current search in the URL, so every screen can return to it. */
export const withSearch = (path: string) => path + window.location.search

export const offerPath = ({ store_id, external_id }: OfferKey) =>
  `${ROUTES.offer}/${encodeURIComponent(store_id)}/${encodeURIComponent(external_id)}`

export function parseOfferPath(path: string): OfferKey | undefined {
  if (!path.startsWith(`${ROUTES.offer}/`)) return undefined
  const [store, id] = path.slice(ROUTES.offer.length + 1).split('/')
  return store && id ? { store_id: decodeURIComponent(store), external_id: decodeURIComponent(id) } : undefined
}
