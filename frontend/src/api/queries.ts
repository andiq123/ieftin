import { type InfiniteData, keepPreviousData, useInfiniteQuery, useQuery } from '@tanstack/react-query'
import type { Filters } from '../lib/filters.ts'
import { isBusy } from '../lib/stores.ts'
import { get } from './client.ts'
import type { SearchResults, Store } from './types.ts'

const PAGE_SIZE = 40
const LIVE_INTERVAL = 2_000
const IDLE_INTERVAL = 15_000
/** While the backend is unreachable, so the app notices quickly when it is back. */
const RETRY_INTERVAL = 5_000
/** How often, and how many times, a search is asked again while stores are still
 *  answering, so their results appear as they arrive. Covers the backend's wait for a store. */
const STREAM_INTERVAL = 1_000
const STREAM_TRIES = 30

/** Stores with how their latest test went. Polled fast while any is being tested,
 *  so the page follows along without a refresh, and slowly otherwise. */
export function useStores() {
  return useQuery({
    queryKey: ['stores'],
    queryFn: () => get<Store[]>('/stores'),
    refetchInterval: ({ state }) =>
      state.errorUpdatedAt > state.dataUpdatedAt ? RETRY_INTERVAL : state.data?.some(isBusy) ? LIVE_INTERVAL : IDLE_INTERVAL,
  })
}

/** Paged search results for the current filters; idle until there is a query.
 *  A search asks every store, so results are kept for a while rather than refetched,
 *  except while stores are still answering: then it keeps asking and they join in. */
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
    refetchInterval: ({ state }) => {
      const pages = (state.data as InfiniteData<SearchResults> | undefined)?.pages
      const loading = pages?.[0]?.sources.some((source) => source.pending)
      return loading && state.dataUpdateCount <= STREAM_TRIES ? STREAM_INTERVAL : false
    },
  })
}
