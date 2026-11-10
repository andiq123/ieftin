import type { CSSProperties, ReactNode } from 'react'
import type { Store } from '../api/types.ts'
import { formatCount, formatEta, relativeTime } from '../lib/format.ts'
import { STATUSES, crawlProgress, isActive, isBusy, isLive, statusOf, totalProducts } from '../lib/stores.ts'
import { type Columns, useSort } from '../lib/useSort.ts'
import { Icon, type IconName } from './Icon.tsx'
import { Segmented } from './Segmented.tsx'

/** The stores page follows the search flow: stores asked live, stores read daily,
 *  and stores that cannot be used right now. */
export function StoresPage({ stores }: { stores: Store[] }) {
  const active = stores.filter(isActive)
  const live = active.filter(isLive)
  const daily = active.filter((store) => !isLive(store))
  const inactive = stores.filter((store) => !isActive(store))

  return (
    <section className="stagger">
      <h1 className="large-title">Stores</h1>

      <div className="status-bar" aria-hidden="true">
        <span className="status-bar__part tone--busy" style={{ flexGrow: live.length }} />
        <span className="status-bar__part tone--ok" style={{ flexGrow: daily.length }} />
        <span className="status-bar__part tone--pending" style={{ flexGrow: inactive.length }} />
      </div>

      <Group icon="bolt" tone="busy" title="Live" count={live.length} note="Asked on every search">
        <ul className="tiles">
          {live.map((store) => (
            <LiveStore key={store.id} store={store} />
          ))}
        </ul>
      </Group>

      <Group
        icon="database"
        tone="ok"
        title="Daily"
        count={daily.length}
        note={`${formatCount(totalProducts(daily))} products, read once a day`}
      >
        <DailyStores stores={daily} />
      </Group>

      <Group icon="empty" tone="pending" title="Inactive" count={inactive.length} note="Not searched, rechecked daily">
        <ul className="list">
          {inactive.map((store) => (
            <InactiveStore key={store.id} store={store} />
          ))}
        </ul>
      </Group>
    </section>
  )
}

interface GroupProps {
  icon: IconName
  tone: string
  title: string
  count: number
  note: string
  children: ReactNode
}

/** One step of the flow: a heading that says what the group is, then its stores. */
function Group({ icon, tone, title, count, note, children }: GroupProps) {
  if (count === 0) return null
  return (
    <section className={`group tone--${tone}`}>
      <header className="group__head">
        <h2>
          <span className="glyph">
            <Icon name={icon} />
          </span>
          {title}
          <span className="group__count">{count}</span>
        </h2>
        <p className="group__note">{note}</p>
      </header>
      {children}
    </section>
  )
}

function StoreLink({ store }: { store: Store }) {
  return (
    <a className="store-link" href={store.url} target="_blank" rel="noreferrer">
      {store.name}
      <Icon name="external" />
    </a>
  )
}

function LiveStore({ store }: { store: Store }) {
  const checking = store.activity.state === 'checking'
  return (
    <li className="tile">
      <StoreLink store={store} />
      <span className="muted">
        {checking ? 'checking…' : store.health_checked_at && relativeTime(store.health_checked_at)}
      </span>
      <Icon name={checking ? 'spinner' : 'check'} className="tone" label={checking ? 'Being checked' : 'Healthy'} />
    </li>
  )
}

type Column = 'name' | 'offers' | 'updated' | 'next'

const COLUMNS: Columns<Store, Column> = {
  name: (store) => store.name,
  offers: (store) => store.offers,
  updated: (store) => store.last_crawl_finished_at,
  next: (store) => store.next_crawl_at,
}

const HEADINGS: { value: Column; label: string }[] = [
  { value: 'name', label: 'Name' },
  { value: 'offers', label: 'Products' },
  { value: 'updated', label: 'Read' },
  { value: 'next', label: 'Next' },
]

function DailyStores({ stores }: { stores: Store[] }) {
  const { sorted, sort, sortBy } = useSort<Store, Column>(stores, COLUMNS, { key: 'offers', direction: 'descending' })
  const options = HEADINGS.map(({ value, label }) => ({
    value,
    label: (
      <>
        {label}
        {sort.key === value && (
          <Icon name={sort.direction === 'ascending' ? 'arrow-up' : 'arrow-down'} label={sort.direction} />
        )}
      </>
    ),
  }))
  return (
    <div className="daily">
      <Segmented label="Sort by" options={options} value={sort.key} onChange={sortBy} />
      <ul className="list">
        {sorted.map((store) => (
          // Named so each row glides to its new place when the order changes.
          <li key={store.id} className="daily__row" style={{ '--vt': `store-${store.id}` } as CSSProperties}>
            <StoreLink store={store} />
            <span className="fact">
              <Icon name="tag" label="Products" />
              {store.offers > 0 ? formatCount(store.offers) : '–'}
            </span>
            <span className="fact">
              <Icon name="clock" label="Prices read" />
              {store.last_crawl_finished_at ? relativeTime(store.last_crawl_finished_at) : '–'}
            </span>
            <NextRead store={store} />
          </li>
        ))}
      </ul>
    </div>
  )
}

/** Live crawl progress while a store is being read, otherwise when the next read is due. */
function NextRead({ store }: { store: Store }) {
  const { activity } = store
  if (activity.state === 'crawling') {
    const total = activity.pages_done + activity.pages_pending
    return (
      <div className="progress" title="The total grows while pages are still being discovered.">
        <div className="progress__track">
          <div className="progress__fill" style={{ width: `${Math.max(2, crawlProgress(store) * 100)}%` }} />
        </div>
        <span className="fact">
          {formatCount(activity.pages_done)} / {formatCount(total)} pages
          {store.eta_seconds !== null && ` · ${formatEta(store.eta_seconds)}`}
        </span>
      </div>
    )
  }
  return (
    <span className="fact">
      <Icon name={isBusy(store) ? 'spinner' : 'crawl'} label="Next read" />
      {isBusy(store) ? 'checking…' : store.next_crawl_at ? relativeTime(store.next_crawl_at) : 'soon'}
    </span>
  )
}

function InactiveStore({ store }: { store: Store }) {
  const status = statusOf(store)
  const { label, icon } = STATUSES[status]
  return (
    <li className={`inactive tone--${status}`} title={store.health_detail ?? undefined}>
      <span className="glyph">
        <Icon name={store.activity.state === 'checking' ? 'spinner' : icon} />
      </span>
      <StoreLink store={store} />
      <span className="muted">{label}</span>
      <span className="fact">
        <Icon name="crawl" label="Next check" />
        {store.next_check_at ? relativeTime(store.next_check_at) : 'queued'}
      </span>
    </li>
  )
}
