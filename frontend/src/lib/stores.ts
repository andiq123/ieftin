import type { Health, Store } from '../api/types.ts'
import type { IconName } from '../components/Icon.tsx'

/** A store's health, plus "pending" for one that has not been checked yet. */
export type Status = Health | 'pending'

/** How each status is named and drawn. */
export const STATUSES: Record<Status, { label: string; icon: IconName }> = {
  ok: { label: 'Active', icon: 'check' },
  pending: { label: 'Awaiting first check', icon: 'clock' },
  no_data: { label: 'No readable prices', icon: 'empty' },
  blocked: { label: 'Blocks automated access', icon: 'lock' },
  unreachable: { label: 'Unreachable', icon: 'offline' },
}

export const statusOf = (store: Store): Status => store.health_status ?? 'pending'

export const isActive = (store: Store) => store.health_status === 'ok'

/** Something is running for this store, or is about to. */
export const isBusy = (store: Store) => store.activity.state !== 'idle' || store.health_status === null

/** Share of the discovered pages already read, 0–1. */
export function crawlProgress(store: Store): number {
  if (store.activity.state !== 'crawling') return 0
  const { pages_done: done, pages_pending: pending } = store.activity
  return done + pending === 0 ? 0 : done / (done + pending)
}

export const isLive = (store: Store) => store.mode === 'live'

export const totalProducts = (stores: Store[]) => stores.reduce((total, store) => total + store.offers, 0)

/** Turns a store id into its name; an unknown id stands in for itself. */
export function storeNamer(stores: Store[]) {
  const names = new Map(stores.map((store) => [store.id, store.name]))
  return (id: string) => names.get(id) ?? id
}
