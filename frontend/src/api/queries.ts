import { keepPreviousData, useInfiniteQuery, useQuery } from '@tanstack/react-query'
import type { Filters } from '../lib/filters.ts'
import type { OfferKey } from '../lib/routes.ts'
import { isBusy } from '../lib/stores.ts'
import { get } from './client.ts'
import type { PriceHistory, SearchResults, Store } from './types.ts'

const PAGE_SIZE = 40
const LIVE_INTERVAL = 2_000
const IDLE_INTERVAL = 30_000
/** While the backend is unreachable, so the app notices quickly when it is back. */
const RETRY_INTERVAL = 5_000

/** Stores with their live state. Polled fast while anything is running, so the
 *  page follows checks and crawls without a refresh, and slowly otherwise. */
export function useStores() {
  return useQuery({
    queryKey: ['stores'],
    queryFn: () => get<Store[]>('/stores'),
    refetchInterval: ({ state }) =>
      state.errorUpdatedAt > state.dataUpdatedAt ? RETRY_INTERVAL : state.data?.some(isBusy) ? LIVE_INTERVAL : IDLE_INTERVAL,
  })
}

/** Paged search results for the current filters; idle until there is a query.
 *  A search asks the live stores, so results are kept for a while rather than refetched. */
export function useSearch(filters: Filters) {
  return useInfiniteQuery({
    queryKey: ['search', filters],
    enabled: filters.q.trim() !== '',
    placeholderData: keepPreviousData,
    staleTime: 5 * 60_000,
    initialPageParam: 0,
    queryFn: ({ pageParam }) =>
      get<SearchResults>('/search', {
        q: filters.q,
        store: filters.store,
        category: filters.category,
        sort: filters.sort,
        in_stock: filters.inStock,
        accessories: filters.accessories,
        limit: PAGE_SIZE,
        offset: pageParam,
      }),
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.offers.length, 0)
      return loaded < last.total ? loaded : undefined
    },
  })
}

/** An offer's recorded price changes, with the offer itself when it is indexed. */
export function useHistory({ store_id, external_id }: OfferKey) {
  return useQuery({
    queryKey: ['history', store_id, external_id],
    queryFn: () => get<PriceHistory>('/history', { store_id, external_id }),
    // Prices change a few times a day at most; the saved list asks once per offer.
    staleTime: 5 * 60_000,
  })
}
