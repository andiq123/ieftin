import type { Health, Store } from '../api/types.ts'
import type { IconName } from '../components/Icon.tsx'

/** A store's health, plus "pending" for one that has not been tested yet. */
export type Status = Health | 'pending'

/** How each status is named and drawn. */
export const STATUSES: Record<Status, { label: string; icon: IconName }> = {
  ok: { label: 'Working', icon: 'check' },
  pending: { label: 'Being tested', icon: 'clock' },
  no_data: { label: 'No readable prices', icon: 'empty' },
  blocked: { label: 'Blocks automated access', icon: 'lock' },
  unreachable: { label: 'Unreachable', icon: 'offline' },
}

export const statusOf = (store: Store): Status => store.status ?? 'pending'

/** Asked on every search: working, or not yet known to be failing. */
export const isSearched = (store: Store) => store.status === 'ok' || store.status === null

/** A test is running for this store, or is about to. */
export const isBusy = (store: Store) => store.checking || store.status === null

/** Turns a store id into its name; an unknown id stands in for itself. */
export function storeNamer(stores: Store[]) {
  const names = new Map(stores.map((store) => [store.id, store.name]))
  return (id: string) => names.get(id) ?? id
}
