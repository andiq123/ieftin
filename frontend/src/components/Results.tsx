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
  const floor = first.accessory_floor_bani
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
            {floor !== null && (
              <span className="fact">
                <Icon name={filters.accessories ? 'eye' : 'eye-off'} />
                {filters.accessories ? 'Accessories shown' : `Accessories under ${formatLei(floor)} lei hidden`}
                <button
                  className="button button--small"
                  type="button"
                  onClick={() => updateFilters({ accessories: !filters.accessories })}
                >
                  {filters.accessories ? 'Hide' : 'Show all'}
                </button>
              </span>
            )}
          </div>
        </div>

        {offers.length === 0 ? (
          <Notice icon="search" title="No offers">
            Try fewer words or remove a filter. New stores may still be indexing.
          </Notice>
        ) : (
          // Keyed by the first page, so a different answer rises in while
          // "show more" extends the list in place.
          <ol className="list offers stagger" key={`${first.total}:${first.offers[0]?.url}`}>
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

/** Says where this answer came from: which stores were asked just now, and which were silent. */
function Sources({ reports, storeName }: SourcesProps) {
  if (reports.length === 0) return null
  const names = (ok: boolean) => reports.filter((report) => report.ok === ok).map((report) => storeName(report.store_id))
  const [answered, silent] = [names(true), names(false)]
  const slowest = Math.max(...reports.map((report) => report.elapsed_ms))
  return (
    <>
      <span className="fact" title={answered.join(', ')}>
        <Icon name="bolt" />
        {answered.length} live{slowest >= 300 && ` in ${(slowest / 1000).toFixed(1)} s`}
      </span>
      <span className="fact">
        <Icon name="database" />
        Rest daily
      </span>
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
