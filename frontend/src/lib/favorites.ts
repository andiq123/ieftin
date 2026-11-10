import { useSyncExternalStore } from 'react'
import type { Offer } from '../api/types.ts'
import { type OfferKey, offerPath } from './routes.ts'

/** An offer the user saved, as it was at that moment: its price then is what
 *  later prices are compared with. Kept in this browser only; no account. */
export interface Favorite {
  offer: Offer
  savedAt: number
}

const STORAGE_KEY = 'ieftin.favorites'
const MAX_FAVORITES = 200

const listeners = new Set<() => void>()
let cached: Favorite[] | undefined

/** Read once and kept, so every reader sees the same list until it changes. */
function read(): Favorite[] {
  if (!cached) {
    try {
      const stored: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '[]')
      cached = Array.isArray(stored) ? (stored as Favorite[]) : []
    } catch {
      cached = []
    }
  }
  return cached
}

function write(favorites: Favorite[]) {
  cached = favorites
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(favorites))
  } catch {
    // Storage is full or blocked: the list still works until the page is closed.
  }
  listeners.forEach((listener) => listener())
}

// Another tab changed the list.
window.addEventListener('storage', (event) => {
  if (event.key !== STORAGE_KEY) return
  cached = undefined
  listeners.forEach((listener) => listener())
})

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Saved offers, newest first. */
export function useFavorites(): Favorite[] {
  return useSyncExternalStore(subscribe, read)
}

/** Whether an offer is saved, as a lookup built once per list. */
export function savedLookup(favorites: Favorite[]) {
  const keys = new Set(favorites.map((favorite) => offerPath(favorite.offer)))
  return (offer: OfferKey) => keys.has(offerPath(offer))
}

/** Saves an offer, or removes it when it is already saved. */
export function toggleFavorite(offer: Offer) {
  const key = offerPath(offer)
  const others = read().filter((favorite) => offerPath(favorite.offer) !== key)
  const removing = others.length < read().length
  write(removing ? others : [{ offer, savedAt: Date.now() }, ...others].slice(0, MAX_FAVORITES))
}
