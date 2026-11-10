import { useCallback, useMemo, useSyncExternalStore } from 'react'
import type { Sort } from '../api/types.ts'
import { replaceUrl, subscribe } from './location.ts'

/** Everything that defines a search. The URL is the single source of truth,
 *  so searches can be shared, bookmarked and navigated with back/forward. */
export interface Filters {
  q: string
  store?: string
  category?: string
  sort: Sort
  inStock: boolean
  accessories: boolean
}

function parse(search: string): Filters {
  const params = new URLSearchParams(search)
  return {
    q: params.get('q') ?? '',
    store: params.get('store') ?? undefined,
    category: params.get('category') ?? undefined,
    sort: params.get('sort') === 'price_desc' ? 'price_desc' : 'price_asc',
    inStock: params.has('stock'),
    accessories: params.has('accessories'),
  }
}

function serialize(filters: Filters): string {
  const params = new URLSearchParams()
  if (filters.q) params.set('q', filters.q)
  if (filters.store) params.set('store', filters.store)
  if (filters.category) params.set('category', filters.category)
  if (filters.sort !== 'price_asc') params.set('sort', filters.sort)
  if (filters.inStock) params.set('stock', '1')
  if (filters.accessories) params.set('accessories', '1')
  const query = params.toString()
  return query ? `?${query}` : window.location.pathname
}

export type UpdateFilters = (patch: Partial<Filters>) => void

export function useFilters(): [Filters, UpdateFilters] {
  const search = useSyncExternalStore(subscribe, () => window.location.search)
  const filters = useMemo(() => parse(search), [search])
  const update = useCallback<UpdateFilters>((patch) => {
    const next = { ...parse(window.location.search), ...patch }
    replaceUrl(serialize(next))
  }, [])
  return [filters, update]
}
