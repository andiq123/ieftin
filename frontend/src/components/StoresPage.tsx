import type { CSSProperties } from 'react'
import type { Store } from '../api/types.ts'
import { relativeTime } from '../lib/format.ts'
import { type Status, STATUSES, isBusy, statusOf } from '../lib/stores.ts'
import { Icon } from './Icon.tsx'

/** Failing stores first, so what needs attention is on top; then by name. */
const ORDER: Status[] = ['unreachable', 'blocked', 'no_data', 'pending', 'ok']
const byUrgency = (a: Store, b: Store) =>
  ORDER.indexOf(statusOf(a)) - ORDER.indexOf(statusOf(b)) || a.name.localeCompare(b.name)

/** The health of every store, as it is right now: the page follows the backend's
 *  tests while they run. Green means the store answers searches. */
export function StoresPage({ stores }: { stores: Store[] }) {
  const sorted = [...stores].sort(byUrgency)
  const slowest = Math.max(1, ...stores.map((store) => store.latency_ms ?? 0))

  return (
    <section className="stagger">
      <h1 className="large-title">Stores</h1>
      {stores.length > 0 && <Summary stores={sorted} />}
      <ul className="list health">
        {sorted.map((store) => (
          <StoreRow key={store.id} store={store} slowest={slowest} />
        ))}
      </ul>
    </section>
  )
}

/** The whole picture in one line, over a strip with one segment per store. */
function Summary({ stores }: { stores: Store[] }) {
  const working = stores.filter((store) => store.status === 'ok').length
  const testing = stores.filter(isBusy).length
  const tested = stores.filter((store) => store.checked_at)
  const latest = tested.map((store) => Date.parse(store.checked_at!)).reduce((a, b) => Math.max(a, b), 0)
  const tone =
    working === stores.length ? 'ok' : working === 0 && testing === 0 ? 'blocked' : testing > 0 ? 'busy' : 'no_data'

  return (
    <header className={`health-summary tone--${tone}`} aria-live="polite">
      <span className={testing > 0 ? 'beacon beacon--busy' : 'beacon'} />
      <div>
        <h2>
          {working === stores.length
            ? `All ${working} stores working`
            : `${working} of ${stores.length} stores working`}
        </h2>
        <p className="muted">
          {testing > 0
            ? `Testing ${testing} ${testing === 1 ? 'store' : 'stores'}…`
            : `Last tested ${relativeTime(latest)} · retested automatically`}
        </p>
      </div>
      <div className="health-strip" aria-hidden="true">
        {stores.map((store) => (
          <span key={store.id} className={`tone--${statusOf(store)}`} title={store.name} />
        ))}
      </div>
    </header>
  )
}

function StoreRow({ store, slowest }: { store: Store; slowest: number }) {
  const status = statusOf(store)
  const working = status === 'ok'
  return (
    <li className={`health__row tone--${status}`}>
      <span className={store.checking ? 'beacon beacon--busy' : 'beacon'} title={STATUSES[status].label} />
      <div className="health__name">
        <a className="store-link" href={store.url} target="_blank" rel="noreferrer">
          {store.name}
          <Icon name="external" />
        </a>
        <span className="muted">
          {store.checking
            ? 'Testing now…'
            : working
              ? `Working · tested ${relativeTime(store.checked_at!)}`
              : STATUSES[status].label}
          {!working && !store.checking && store.next_check_at && ` · retry ${relativeTime(store.next_check_at)}`}
        </span>
      </div>
      {working && store.latency_ms !== null && (
        <div className="health__speed" title="How long the store took to answer the test search">
          <span className="fact">{formatLatency(store.latency_ms)}</span>
          <span className="meter" style={{ '--share': store.latency_ms / slowest } as CSSProperties} />
        </div>
      )}
      {!working && store.detail && !store.checking && <span className="health__reason muted">{store.detail}</span>}
    </li>
  )
}

const formatLatency = (ms: number) => (ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s`)
