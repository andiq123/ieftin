import type { Facet, SearchResults } from '../api/types.ts'
import type { Filters as FiltersState, UpdateFilters } from '../lib/filters.ts'
import { formatLei } from '../lib/format.ts'
import { Chip } from './Chip.tsx'
import { Icon } from './Icon.tsx'
import { Segmented } from './Segmented.tsx'

const SORTS = [
  {
    value: 'price_asc',
    label: (
      <>
        <Icon name="arrow-down" />
        Lowest
      </>
    ),
  },
  {
    value: 'price_desc',
    label: (
      <>
        <Icon name="arrow-up" />
        Highest
      </>
    ),
  },
] as const

interface Props {
  filters: FiltersState
  updateFilters: UpdateFilters
  results: SearchResults
  storeName: (id: string) => string
}

export function Filters({ filters, updateFilters, results, storeName }: Props) {
  return (
    <div className="filters">
      <div className="filters__row">
        <Segmented label="Sort by price" options={SORTS} value={filters.sort} onChange={(sort) => updateFilters({ sort })} />
        <Chip selected={filters.inStock} onClick={() => updateFilters({ inStock: !filters.inStock })}>
          <Icon name="check" />
          In stock
        </Chip>
      </div>
      <FacetChips
        label="Store"
        facets={results.stores}
        selected={filters.store}
        onSelect={(store) => updateFilters({ store })}
        name={storeName}
      />
      <FacetChips
        label="Category"
        facets={results.categories}
        selected={filters.category}
        onSelect={(category) => updateFilters({ category })}
      />
    </div>
  )
}

interface FacetChipsProps {
  label: string
  facets: Facet[]
  selected: string | undefined
  onSelect: (key: string | undefined) => void
  name?: (key: string) => string
}

function FacetChips({ label, facets, selected, onSelect, name = (key) => key }: FacetChipsProps) {
  if (facets.length < 2 && !selected) return null
  return (
    <div className="filters__row" role="group" aria-label={label}>
      {facets.map((facet) => {
        const from = `${formatLei(facet.min_price_bani)} lei`
        return (
          <Chip
            key={facet.key}
            selected={facet.key === selected}
            title={`${facet.count} offers from ${from}`}
            onClick={() => onSelect(facet.key === selected ? undefined : facet.key)}
          >
            {name(facet.key)}
            <span className="chip__meta">
              {facet.count} · {from}
            </span>
          </Chip>
        )
      })}
    </div>
  )
}
