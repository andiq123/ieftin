import { ApiError } from '../api/client.ts'
import { useSearch } from '../api/queries.ts'
import type { SourceReport, Store } from '../api/types.ts'
import type { Filters as FiltersState, UpdateFilters } from '../lib/filters.ts'
import { savedLookup, useFavorites } from '../lib/favorites.ts'
import { formatCount, formatLei } from '../lib/format.ts'
import { storeNamer } from '../lib/stores.ts'
import { Filters } from './Filters.tsx'
import { Icon } from './Icon.tsx'
import { Notice } from './Notice.tsx'
import { OfferRow } from './OfferRow.tsx'

interface Props {
  filters: FiltersState
  updateFilters: UpdateFilters
  stores: Store[]
}

export function Results({ filters, updateFilters, stores }: Props) {
  const search = useSearch(filters)
  const isSaved = savedLookup(useFavorites())

  if (search.isError) {
    const limited = search.error instanceof ApiError && search.error.status === 429
    return (
      <Notice
        icon={limited ? 'clock' : 'alert'}
        title={limited ? 'Too many searches' : 'Search failed'}
        alert
        action={{ label: 'Try again', onClick: () => search.refetch() }}
      >
        {limited ? 'Wait a minute, then search again.' : 'Ieftin could not be reached. Try again in a moment.'}
      </Notice>
    )
  }
  if (!search.data) {
    return (
      <section className="results" aria-busy="true">
        <Searching query={filters.q} />
        <ol className="list offers offers--skeleton" aria-hidden="true">
          {Array.from({ length: 6 }, (_, row) => (
            <li key={row} />
          ))}
        </ol>
      </section>
    )
  }

  const [first] = search.data.pages
  const offers = search.data.pages.flatMap((page) => page.offers)
  const storeName = storeNamer(stores)
  const cheapestBani = filters.sort === 'price_asc' ? offers[0]?.price_bani : undefined
  const { accessories_total: accessories, accessories_hidden: hidden } = first
  // Nothing but accessories matched, so they are shown instead of an empty page.
  const onlyAccessories = accessories > 0 && hidden === 0 && !filters.accessories
  // A new search is running while the previous results are still on screen.
  const changing = search.isPlaceholderData

  return (
    <section className="results" aria-busy={changing}>
      {changing && <Searching query={filters.q} />}
      <div className={changing ? 'results__body results__body--changing' : 'results__body'} inert={changing}>
        <Filters filters={filters} updateFilters={updateFilters} results={first} storeName={storeName} />

        <div className="results__summary">
          <h2 aria-live="polite">
            {formatCount(first.total)} {first.total === 1 ? 'offer' : 'offers'}
          </h2>
          <div className="facts">
            <Sources reports={first.sources} storeName={storeName} />
            {onlyAccessories ? (
              <span className="fact">
                <Icon name="eye" />
                Only accessories found for this search
              </span>
            ) : (
              accessories > 0 && (
                <span className="fact">
                  <Icon name={filters.accessories ? 'eye' : 'eye-off'} />
                  {filters.accessories
                    ? `${formatCount(accessories)} accessories shown`
                    : `${formatCount(hidden)} ${hidden === 1 ? 'accessory' : 'accessories'} hidden`}
                  <button
                    className="button button--small"
                    type="button"
                    onClick={() => updateFilters({ accessories: !filters.accessories })}
                  >
                    {filters.accessories ? 'Hide' : 'Show all'}
                  </button>
                </span>
              )
            )}
          </div>
        </div>

        {offers.length === 0 ? (
          first.sources.some((source) => source.pending) ? (
            <Notice icon="spinner" title="Nothing yet">
              Some stores are still answering.
            </Notice>
          ) : (
            <Notice icon="search" title="No offers">
              Try fewer words or remove a filter.
            </Notice>
          )
        ) : (
          // Keyed by the search, so a different search rises in, while offers that
          // arrive from slower stores and "show more" extend the list in place.
          <ol className="list offers stagger" key={JSON.stringify(filters)}>
            {offers.map((offer, index) => {
              // Ties share the lowest price, but only the first row carries the price tag.
              const cheapest = cheapestBani !== undefined && index === 0
              const extra = cheapestBani === undefined ? 0 : offer.price_bani - cheapestBani
              return (
                <OfferRow
                  key={offer.url}
                  offer={offer}
                  storeName={storeName(offer.store_id)}
                  saved={isSaved(offer)}
                  rank={index + 1}
                  cheapest={cheapest}
                  note={
                    cheapest ? (
                      <>
                        <Icon name="tag" />
                        Cheapest
                      </>
                    ) : (
                      extra > 0 && `+${formatLei(extra)} lei`
                    )
                  }
                />
              )
            })}
          </ol>
        )}

        {search.hasNextPage && (
          <button
            className="button results__more"
            type="button"
            disabled={search.isFetchingNextPage}
            onClick={() => search.fetchNextPage()}
          >
            {search.isFetchingNextPage && <Icon name="spinner" />}
            Show more
          </button>
        )}
      </div>
    </section>
  )
}

interface SourcesProps {
  reports: SourceReport[]
  storeName: (id: string) => string
}

/** Says where this answer came from: how many stores answered, which are still
 *  loading in the background, and which were silent. */
function Sources({ reports, storeName }: SourcesProps) {
  if (reports.length === 0) return null
  const names = (pick: (report: SourceReport) => boolean) =>
    reports.filter(pick).map((report) => storeName(report.store_id))
  const answered = names((report) => report.ok)
  const loading = names((report) => report.pending)
  const silent = names((report) => !report.ok && !report.pending)
  const slowest = Math.max(0, ...reports.filter((report) => report.ok).map((report) => report.elapsed_ms))
  return (
    <>
      <span className="fact" title={answered.join(', ')}>
        <Icon name="bolt" />
        {loading.length > 0
          ? `${answered.length} of ${reports.length} stores answered`
          : `${answered.length} ${answered.length === 1 ? 'store' : 'stores'} asked just now`}
        {loading.length === 0 && slowest >= 300 && ` in ${(slowest / 1000).toFixed(1)} s`}
      </span>
      {loading.length > 0 && (
        <span className="fact">
          <Icon name="spinner" />
          Still loading: {loading.length > 4 ? `${loading.length} stores` : loading.join(', ')}
        </span>
      )}
      {silent.length > 0 && (
        <span className="fact fact--warn">
          <Icon name="alert" />
          No answer: {silent.join(', ')}
        </span>
      )}
    </>
  )
}

/** The status pill shown while the stores are being asked. */
function Searching({ query }: { query: string }) {
  return (
    <div className="searching">
      <p role="status">
        <Icon name="spinner" />
        Asking stores for “{query}”…
      </p>
    </div>
  )
}
